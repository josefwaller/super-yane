use std::time::Duration;

use log::*;
use spc700::{HasAddressBus as Spc700AddressBuss, IPL, Processor as Spc700};
use wdc65816::{HasAddressBus, Processor};

use crate::{
    Cartridge, InputPort, Ppu,
    dma::{AddressAdjustMode as DmaAddressAdjustMode, Channel as DmaChannel},
};
use paste::paste;

// Contains everything except the processor(s)
#[derive(Clone)]
pub struct ExternalArchitecture {
    pub ram: [u8; 0x20000],
    pub spc_ram: [u8; 0x10000],
    pub cpu_to_apu_reg: [u8; 4],
    pub apu_to_cpu_reg: [u8; 4],
    pub expose_ipl_rom: bool,
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    /// DMA Channels
    pub dma_channels: [DmaChannel; 8],
    pub input_ports: [InputPort; 2],
    total_master_clocks: u64,
    apu_master_clocks: u64,
    open_bus_value: u8,
    nmi_enabled: bool,
}

// Todo: move somewhere
fn byte_from_bits(bits: [bool; 8]) -> u8 {
    bits.iter()
        .enumerate()
        .map(|(i, b)| u8::from(*b) << i)
        .sum()
}
impl ExternalArchitecture {
    // Reads a byte without advancing anything
    // Returns the value, and how many master clocks were needed to access the memory
    pub fn read_byte(&self, addr: usize) -> (u8, u32) {
        if (0x7E0000..0x800000).contains(&addr) {
            (self.ram[addr - 0x7E0000], 12)
        } else if addr < 0x400000 {
            let a = addr & 0xFFFF;
            if a < 0x2000 {
                (self.ram[a], 12)
            } else if a < 0x2100 {
                (0, 12)
            } else if a < 0x2140 {
                (self.ppu.read_byte(a), 12)
            } else if a < 0x2144 {
                (self.apu_to_cpu_reg[a - 0x2140], 12)
            } else if a >= 0x4218 && a < 0x4220 {
                // Read controller data
                let i = (a / 2) % 2;
                let j = a % 2;
                let input_port = self.input_ports[i];
                match input_port {
                    InputPort::Empty => (self.open_bus_value, 12),
                    InputPort::StandardController {
                        a,
                        b,
                        x,
                        y,
                        up,
                        left,
                        right,
                        down,
                        start,
                        select,
                        r,
                        l,
                    } => {
                        match j {
                            // Low byte
                            0 => (byte_from_bits([false, false, false, false, r, l, x, a]), 12),
                            1 => (
                                byte_from_bits([right, left, down, up, start, select, y, b]),
                                12,
                            ),
                            _ => panic!("Should be impossible"),
                        }
                    }
                }
            } else if a < 0x8000 {
                (self.ppu.read_byte(a), 12)
            } else {
                (self.cartridge.read_byte(addr), 12)
            }
        } else {
            (self.cartridge.read_byte(addr), 12)
        }
    }
    // Writes a byte without advancing anything
    // May trigger a DMA
    // Returns the number of master cycles needed to access the memory
    pub fn write_byte(&mut self, addr: usize, value: u8) -> u32 {
        if (0x7E0000..0x800000).contains(&addr) {
            self.ram[addr - 0x7E0000] = value;
            12
        } else {
            let a = addr % (0x800000);
            // Check for non-rom area
            if a < 0x400000 && a & 0xFFFF < 0x8000 {
                let a = a % 0x8000;
                match a {
                    (0..0x2000) => {
                        self.ram[a] = value;
                        12
                    }
                    (0x2000..0x2100) => {
                        // Open bus?
                        12
                    }
                    (0x2100..0x2140) => {
                        // PPU Registers
                        self.ppu.write_byte(a, value);
                        12
                    }
                    (0x2140..0x2144) => {
                        self.cpu_to_apu_reg[a - 0x2140] = value;
                        12
                    }
                    0x4200 => {
                        self.nmi_enabled = (value & 0x80) != 0;
                        12
                    }
                    0x420B => {
                        (0..8).for_each(|i| {
                            if (value >> i) & 0x01 != 0 {
                                let mut d = self.dma_channels[i].clone();
                                let mut bytes_transferred = 0;
                                // Todo: handling timing of DMA
                                loop {
                                    let src = d.src_bank as usize * 0x10000 + d.src_addr as usize;
                                    let dest = d.dest_addr
                                        + d.transfer_pattern
                                            [bytes_transferred % d.transfer_pattern.len()]
                                            as usize;
                                    let v = HasAddressBus::read(self, src);
                                    HasAddressBus::write(self, dest, v);
                                    let md = &mut self.dma_channels[i];
                                    bytes_transferred += 1;
                                    match md.adjust_mode {
                                        DmaAddressAdjustMode::Increment => {
                                            md.src_addr = d.src_addr.wrapping_add(1)
                                        }
                                        DmaAddressAdjustMode::Decrement => {
                                            md.src_addr = d.src_addr.wrapping_sub(1)
                                        }
                                        _ => {}
                                    }
                                    md.byte_counter = md.byte_counter.wrapping_sub(1);
                                    if md.byte_counter == 0 {
                                        break;
                                    }
                                    d = md.clone();
                                }
                                // todo remove
                                self.ppu.vram_addr = 0;
                            }
                        });
                        // Todo: determine how many clock cycles consumed
                        return 12;
                    }
                    0x420C => {
                        debug!("Value written to HDMA enable: {:02X}", value);
                        12
                    }
                    0x4300..0x4308 => {
                        let lsb = a & 0x0F;
                        let r = (a & 0xF0) >> 4;
                        if r > 7 {
                            12
                        } else {
                            let d = &mut self.dma_channels[r];
                            // DMA register
                            if lsb == 0 {
                                d.transfer_pattern = match (value & 0x07) {
                                    0 => vec![0],
                                    1 => vec![0, 1],
                                    2 | 6 => vec![0; 2],
                                    3 | 7 => vec![0, 0, 1, 1],
                                    4 => vec![0, 1, 2, 3],
                                    5 => vec![0, 1, 0, 1],
                                    _ => panic!("Should be impossible {:X}", (value & 0x07)),
                                };
                                d.adjust_mode = match value & 0x18 {
                                    0x00 => DmaAddressAdjustMode::Increment,
                                    0x10 => DmaAddressAdjustMode::Decrement,
                                    _ => DmaAddressAdjustMode::Fixed,
                                };
                                d.indirect = (value & 0x40) != 0;
                                d.direction = (value & 0x80) != 0;
                            } else if lsb == 1 {
                                d.dest_addr = 0x2100 + value as usize;
                            } else if lsb == 2 {
                                // Address low byte
                                d.src_addr = (d.src_addr & 0xFF00) | value as u16;
                            } else if lsb == 3 {
                                // Address high byte
                                d.src_addr = (d.src_addr & 0x00FF) | (value as u16 * 0x100);
                            } else if lsb == 4 {
                                // Address bank
                                d.src_bank = value;
                            } else if lsb == 5 {
                                // Byte counter low byte
                                d.byte_counter = (d.byte_counter & 0xFF00) | value as u16;
                            } else if lsb == 6 {
                                // Byte counter high byte
                                d.byte_counter = (d.byte_counter & 0x00FF) | (value as u16 * 0x100);
                            }
                            12
                        }
                    }
                    _ => 12,
                }
            } else {
                // self.cartridge.read_byte(a)
                12
            }
        }
    }
    fn advance(&mut self, master_clocks: u32) {
        self.total_master_clocks += master_clocks as u64;
        self.ppu.advance_master_clock(master_clocks)
    }
    pub fn read_apu(&self, address: usize) -> u8 {
        match address {
            0x00F4..0x00F8 => {
                // debug!("APU reading from CPU {:04X}", address);
                self.cpu_to_apu_reg[address - 0x00F4]
            }
            0x0000..0xFFC0 => self.spc_ram[address],
            0xFFC0..0x10000 => {
                if self.expose_ipl_rom {
                    IPL[address - 0xFFC0]
                } else {
                    self.spc_ram[address]
                }
            }
            _ => panic!("Should be impossible"),
        }
    }
}
impl HasAddressBus for ExternalArchitecture {
    fn io(&mut self) {
        self.advance(6);
    }
    fn read(&mut self, address: usize) -> u8 {
        // Todo find a better solution
        if address & 0x800000 < 0x400000 && address & 0xFFFF == 0x4210 {
            self.ppu.read_byte_mut(address, self.open_bus_value)
        } else {
            let (v, clks) = self.read_byte(address);
            self.advance(clks);
            self.open_bus_value = v;
            v
        }
    }
    fn write(&mut self, address: usize, value: u8) {
        let clks = self.write_byte(address, value);
        self.advance(clks);
    }
}
impl Spc700AddressBuss for ExternalArchitecture {
    fn io(&mut self) {
        self.apu_master_clocks += 1;
    }
    fn read(&mut self, address: usize) -> u8 {
        self.apu_master_clocks += 1;
        self.read_apu(address)
    }
    fn write(&mut self, address: usize, value: u8) {
        self.apu_master_clocks += 1;
        match address {
            0x00F1 => {
                self.expose_ipl_rom = (value & 0x80) != 0;
                if value & 0x10 != 0 {
                    self.cpu_to_apu_reg[0] = 0x00;
                    self.cpu_to_apu_reg[1] = 0x00;
                }
                if value & 0x20 != 0 {
                    self.cpu_to_apu_reg[2] = 0x00;
                    self.cpu_to_apu_reg[3] = 0x00;
                }
            }
            0x00F4..0x00F8 => {
                // debug!("APU writing {:02X} to CPU {:04X}", value, address);
                self.apu_to_cpu_reg[address - 0x00F4] = value
            }
            _ => self.spc_ram[address] = value,
        }
    }
}

/// The entire S.N.E.S. Console
#[derive(Clone)]
pub struct Console {
    // The processor is the driving force behind the emulator, so the console is split into 2 parts
    // so that the CPU can be passed the rest of the console, and advance it in sync
    // i.e. cpu.advance_instructions(&rest) will advacne both by 3 CPU instructions
    // Since this is a bit of a weird structure we provide methods to get the different parts of the console as mutable/immuatable
    // references
    cpu: Processor,
    apu: Spc700,
    rest: ExternalArchitecture,
}

// Expose a field in the `rest` struct via a console method
macro_rules! rest_field {
    ($field: ident, $type: ty) => {
        paste! {
            pub fn [<$field _mut>](&mut self) -> &mut $type {
                &mut self.rest.$field
            }
            pub fn $field(&self) -> &$type {
                &self.rest.$field
            }
        }
    };
}
impl Console {
    rest_field! {ppu, Ppu}
    rest_field! {ram, [u8; 0x20000]}
    rest_field! {cartridge, Cartridge}
    rest_field! {dma_channels, [DmaChannel; 8]}
    rest_field! {total_master_clocks, u64}
    rest_field! {input_ports, [InputPort; 2]}
    rest_field! {apu_to_cpu_reg, [u8; 4]}
    rest_field! {cpu_to_apu_reg, [u8; 4]}
    pub fn cpu(&self) -> &Processor {
        &self.cpu
    }
    pub fn cpu_mut(&mut self) -> &mut Processor {
        &mut self.cpu
    }
    pub fn apu(&self) -> &Spc700 {
        &self.apu
    }
    pub fn apu_opcode(&self) -> u8 {
        0
    }
    pub fn with_cartridge(cartridge_data: &[u8]) -> Console {
        let mut c = Console {
            cpu: Processor::default(),
            apu: Spc700::default(),
            rest: ExternalArchitecture {
                ram: [0; 0x20000],
                cpu_to_apu_reg: [0; 4],
                apu_to_cpu_reg: [0; 4],
                expose_ipl_rom: true,
                spc_ram: [0; 0x10000],
                cartridge: Cartridge::from_data(cartridge_data),
                input_ports: [InputPort::default_standard_controller(); 2],
                ppu: Ppu::default(),
                dma_channels: core::array::from_fn(|_| DmaChannel::default()),
                total_master_clocks: 0,
                apu_master_clocks: 0,
                open_bus_value: 0,
                nmi_enabled: false,
            },
        };
        c.cpu.pc =
            c.cartridge().read_byte(0xFFFC) as u16 + 0x100 * c.cartridge().read_byte(0xFFFD) as u16;
        debug!("Initialized PC to {:X}", c.cpu.pc);
        c
    }
    pub fn advance_instructions(&mut self, num_instructions: u32) {
        self.advance_instructions_with_hooks(num_instructions, &mut |_| {}, &mut |_| {});
    }
    pub fn advance_instructions_with_hooks(
        &mut self,
        num_instructions: u32,
        before_cpu_step: &mut dyn FnMut(&Console),
        before_apu_step: &mut dyn FnMut(&Console),
    ) {
        (0..num_instructions).for_each(|_| {
            let vblank = self.ppu().vblank;
            before_cpu_step(&self);
            self.cpu.step(&mut self.rest);
            if !vblank && self.ppu().vblank && self.rest.nmi_enabled {
                self.cpu.on_nmi(&mut self.rest);
            }
            while self.rest.apu_master_clocks * 1_000_000 / 1_024_000
                < self.rest.total_master_clocks * 1_000_000_000 / 21_477_000_000
            {
                // Catch up the APU
                before_apu_step(&self);
                self.apu.step(&mut self.rest);
            }
        });
    }
    pub fn advance_until(&mut self, should_stop: &mut impl FnMut(&Console) -> bool) -> u32 {
        std::iter::from_fn(|| {
            if should_stop(&self) {
                None
            } else {
                self.cpu.step(&mut self.rest);
                self.apu.step(&mut self.rest);
                Some(1)
            }
        })
        .sum()
    }
    /// Get the opcode that the console will execute on the next call to [`Console::advance_instructions``]
    pub fn opcode(&self) -> u8 {
        self.rest.read_byte(self.pc()).0
    }
    /// Get the current program counter of the console
    pub fn pc(&self) -> usize {
        self.cpu.pbr as usize * 0x10000 + self.cpu.pc as usize
    }
    /// Reset the console
    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.rest);
    }
    /// Return [`true`] if the console is currently in VBlank, and [`false`] otherwise
    pub fn in_vblank(&self) -> bool {
        // Todo: Actually implement
        self.ppu().vblank
    }
    /// Read a byte in CPU space
    pub fn read_byte_cpu(&self, address: usize) -> u8 {
        self.rest.read_byte(address).0
    }
    /// Read a byte in APU space
    pub fn read_byte_apu(&self, address: usize) -> u8 {
        self.rest.read_apu(address)
    }
}
