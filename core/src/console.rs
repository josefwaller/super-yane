use std::time::Duration;

use log::*;
use spc700::{HasAddressBus as Spc700AddressBuss, IPL, Processor as Spc700};
use wdc65816::{HasAddressBus, Processor};

use crate::{
    Cartridge, Cpu, InputPort, Ppu,
    dma::{AddressAdjustMode as DmaAddressAdjustMode, Channel as DmaChannel},
    math::Math,
};
use paste::paste;

#[derive(Debug, Clone, Copy, Default)]
pub struct ApuTimer {
    pub timer: u8,
    /// Really u4
    pub counter: u8,
}

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
    // Math module for multiplication and division
    pub math: Math,
    /// DMA Channels
    pub dma_channels: [DmaChannel; 8],
    pub input_ports: [InputPort; 2],
    total_master_clocks: u64,
    total_apu_clocks: u64,
    open_bus_value: u8,
    nmi_enabled: bool,
    pub timers: [ApuTimer; 3],
    /// Whether fast ROM access is enabled through MEMSEL
    fast_rom_enabled: bool,
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
            (self.ram[addr - 0x7E0000], 8)
        } else if addr < 0x400000 {
            let a = addr & 0xFFFF;
            match a {
                0x0000..0x2000 => (self.ram[a], 6),
                0x2000..0x2100 => (0, 6),
                0x2100..0x2140 => (self.ppu.read_byte(a), 6),
                0x2140..0x2180 => (self.apu_to_cpu_reg[a % 4], 6),
                0x4002..=0x4006 | 0x4214..=0x4217 => (self.math.read_byte(a), 6),
                0x4218..0x4220 => {
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
                                _ => unreachable!("Should be impossible"),
                            }
                        }
                    }
                }
                0x4220..0x8000 => (self.ppu.read_byte(a), 6),
                0x8000..=0xFFFF => (self.cartridge.read_byte(addr), 8),
                _ => (0, 6),
            }
        } else {
            (
                self.cartridge.read_byte(addr),
                if addr > 0x800000 && self.fast_rom_enabled {
                    6
                } else {
                    8
                },
            )
        }
    }
    // Writes a byte without advancing anything
    // May trigger a DMA
    // Returns the number of master cycles needed to access the memory
    pub fn write_byte(&mut self, addr: usize, value: u8) -> u32 {
        if (0x7E0000..0x800000).contains(&addr) {
            self.ram[addr - 0x7E0000] = value;
            8
        } else {
            let a = addr % (0x800000);
            // Check for non-rom area
            if a < 0x400000 && a & 0xFFFF < 0x8000 {
                let a = a % 0x8000;
                match a {
                    (0..0x2000) => {
                        self.ram[a] = value;
                        8
                    }
                    (0x2000..0x2100) => {
                        // Open bus?
                        6
                    }
                    (0x2100..0x2140) => {
                        // PPU Registers
                        self.ppu.write_byte(a, value);
                        6
                    }
                    (0x2140..0x2180) => {
                        self.cpu_to_apu_reg[a % 4] = value;
                        6
                    }
                    0x4200 => {
                        self.nmi_enabled = (value & 0x80) != 0;
                        6
                    }
                    0x420B => {
                        (0..8).for_each(|i| {
                            if (value >> i) & 0x01 != 0 {
                                self.dma_channels[i].is_executing = true;
                                self.dma_channels[i].num_bytes_transferred = 0;
                            }
                        });
                        return 6;
                    }
                    0x420C => {
                        (0..8).for_each(|i| {
                            if (value >> i) & 0x01 != 0 {
                                let d = &mut self.dma_channels[i];
                                d.hdma_enable = true;
                            }
                        });
                        6
                    }
                    0x4202..=0x4206 | 0x4214..=0x4217 => {
                        self.math.write_byte(a, value);
                        6
                    }
                    0x420D => {
                        self.fast_rom_enabled = value & 0x01 != 0;
                        6
                    }
                    0x4300..0x43F8 => {
                        let lsb = a & 0x0F;
                        let r = (a & 0xF0) >> 4;
                        if r < 8 {
                            let d = &mut self.dma_channels[r];
                            // DMA register
                            match lsb {
                                0 => {
                                    d.transfer_pattern = match value & 0x07 {
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
                                }
                                1 => {
                                    d.dest_addr = 0x2100 + value as usize;
                                }
                                2 => {
                                    // Address low byte
                                    d.src_addr = (d.src_addr & 0xFF00) | value as u16;
                                    d.hdma_table_addr = d.src_addr;
                                }
                                3 => {
                                    // Address high byte
                                    d.src_addr = (d.src_addr & 0x00FF) | (value as u16 * 0x100);
                                    d.hdma_table_addr = d.src_addr;
                                }
                                4 => {
                                    // Address bank
                                    d.src_bank = value;
                                }
                                5 => {
                                    // Byte counter low byte
                                    d.byte_counter = (d.byte_counter & 0xFF00) | value as u16;
                                }
                                6 => {
                                    // Byte counter high byte
                                    d.byte_counter =
                                        (d.byte_counter & 0x00FF) | (value as u16 * 0x100);
                                }
                                7 => {
                                    d.hdma_bank = value;
                                }
                                8 => {
                                    d.current_hdma_table_addr =
                                        (d.current_hdma_table_addr & 0xFF00) + value as u16;
                                }
                                9 => {
                                    d.current_hdma_table_addr = (value as u16) * 0x100
                                        + (d.current_hdma_table_addr & 0x00FF);
                                }
                                0xA => match value {
                                    0 => {}
                                    1..=0x80 => {
                                        d.hdma_line_counter = value;
                                        d.hdma_repeat = false;
                                    }
                                    0x81..=0xFF => {
                                        d.hdma_line_counter = value - 0x80;
                                        d.hdma_repeat = true
                                    }
                                },
                                _ => {}
                            }
                        }
                        6
                    }
                    _ => 6,
                }
            } else {
                if addr >= 0x800000 && self.fast_rom_enabled {
                    6
                } else {
                    8
                }
            }
        }
    }
    fn advance(&mut self, master_clocks: u32) {
        self.total_master_clocks += master_clocks as u64;
        self.ppu.advance_master_clock(master_clocks);
    }
    fn advance_apu(&mut self, apu_clocks: u64) {
        // Todo maybe: clean this up
        (0..apu_clocks).for_each(|_| {
            self.total_apu_clocks += 1;
            // Clock the timers every 16 (timers 0 and 1) or 128 (timer 2) APU cycles
            [16, 16, 128].into_iter().enumerate().for_each(|(i, clks)| {
                if self.total_apu_clocks % clks == 0 {
                    // Increment timer and increment counter if it overflows
                    let t = self.timers[i].timer.wrapping_add(1);
                    if t < self.timers[i].timer {
                        self.timers[i].counter = self.timers[i].counter.wrapping_add(1);
                    }
                    self.timers[i].timer = t;
                }
            });
        });
    }
    pub fn read_apu(&self, address: usize) -> u8 {
        match address {
            0x00F4..0x00F8 => self.cpu_to_apu_reg[address - 0x00F4],
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
        // really need todo
        if address & 0x800000 < 0x400000
            && (address & 0xFFFF == 0x2139 || address & 0xFFFF == 0x4210)
        {
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
        self.advance_apu(1);
    }
    fn read(&mut self, address: usize) -> u8 {
        self.advance_apu(1);
        match address {
            0x00FD..0x00FF => {
                let v = self.timers[address - 0x00FD].counter;
                self.timers[address - 0x00FD].counter = 0;
                v
            }
            _ => self.read_apu(address),
        }
    }
    fn write(&mut self, address: usize, value: u8) {
        self.advance_apu(1);
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
            0x00F4..0x00F8 => self.apu_to_cpu_reg[address - 0x00F4] = value,
            0x00FA..0x00FD => {
                self.timers[address - 0x00FA].timer = value;
            }
            _ => self.spc_ram[address] = value,
        }
    }
}

/// The entire S.N.E.S. Console
#[derive(Clone)]
pub struct Console {
    /// The CPU is the driving force of the console.
    /// It advances the rest of the console through read and write methods in rest.
    cpu: Cpu,
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
    rest_field! {total_apu_clocks, u64}
    rest_field! {input_ports, [InputPort; 2]}
    rest_field! {apu_to_cpu_reg, [u8; 4]}
    rest_field! {cpu_to_apu_reg, [u8; 4]}
    pub fn cpu(&self) -> &Processor {
        &self.cpu.core
    }
    pub fn cpu_mut(&mut self) -> &mut Processor {
        &mut self.cpu.core
    }
    pub fn apu(&self) -> &Spc700 {
        &self.apu
    }
    pub fn apu_opcode(&self) -> u8 {
        0
    }
    pub fn with_cartridge(cartridge_data: &[u8]) -> Console {
        let mut c = Console {
            cpu: Cpu::default(),
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
                total_apu_clocks: 0,
                open_bus_value: 0,
                nmi_enabled: false,
                timers: [ApuTimer::default(); 3],
                fast_rom_enabled: false,
                math: Math::default(),
            },
        };
        c.cpu.reset(&mut c.rest);
        debug!("Initialized PC to {:X}", c.cpu.core.pc);
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
            let vblank = self.ppu().is_in_vblank();
            let hblank = self.ppu().is_in_hblank() && !vblank;
            before_cpu_step(&self);
            self.cpu.step(&mut self.rest);
            if !vblank && self.ppu().is_in_vblank() {
                // Trigger NMI
                if self.rest.nmi_enabled {
                    self.cpu.on_nmi(&mut self.rest);
                }
                // Disable all HDMA channels
                self.rest.dma_channels.iter_mut().for_each(|d| {
                    d.hdma_enable = false;
                    d.hdma_line_counter = 0;
                });
            }
            let scanline = self.ppu().scanline();
            // the timing here is maybe a little bit off, but if we just exited vblank, set up the hblank DMA registers
            if vblank && !self.ppu().is_in_vblank() {
                (0..self.rest.dma_channels.len()).for_each(|i| {
                    macro_rules! d {
                        () => {
                            self.rest.dma_channels[i]
                        };
                    }
                    // Set line counter to 0 (will trigger an HDMA at scanline 0 if the channel is enabled)
                    d!().hdma_line_counter = 0;
                    // Copy table address
                    d!().current_hdma_table_addr = d!().hdma_table_addr;
                });
            }
            if !vblank && !hblank && self.ppu().is_in_hblank() {
                // Trigger HDMAs
                (0..self.rest.dma_channels.len()).for_each(|i| {
                    let mut d = self.rest.dma_channels[i].clone();
                    if d.hdma_enable {
                        match d.hdma_line_counter {
                            0 => {
                                // Read next byte
                                match self.rest.read_byte(d.hdma_table_addr()).0 {
                                    0 => d.hdma_enable = false,
                                    lc => {
                                        // Get line counter
                                        match lc {
                                            0 => unreachable!(),
                                            0x01..=0x80 => {
                                                d.hdma_repeat = false;
                                                d.hdma_line_counter = lc;
                                            }
                                            0x81..=0xFF => {
                                                d.hdma_repeat = true;
                                                d.hdma_line_counter = lc - 0x80;
                                            }
                                        }
                                        // Copy values
                                        let table_addr = d.current_hdma_table_addr;
                                        // Set up DMA values
                                        if d.indirect {
                                            d.indirect_data_addr = u16::from_le_bytes([
                                                self.rest
                                                    .read_byte(table_addr.wrapping_add(1) as usize)
                                                    .0,
                                                self.rest
                                                    .read_byte(table_addr.wrapping_add(2) as usize)
                                                    .0,
                                            ]);
                                            d.current_hdma_table_addr = table_addr.wrapping_add(3);
                                        } else {
                                            d.src_addr = table_addr.wrapping_add(1);
                                            d.current_hdma_table_addr = table_addr
                                                .wrapping_add(1 + d.transfer_pattern.len() as u16);
                                        }
                                        // Trigger DMA
                                        d.is_executing = true;
                                        d.num_bytes_transferred = 0;
                                        // Since we just went over a scanline here, dec line counter
                                        d.hdma_line_counter -= 1;
                                    }
                                }
                            }
                            _ => {
                                d.hdma_line_counter -= 1;
                                if d.hdma_repeat {
                                    d.is_executing = true;
                                    d.num_bytes_transferred = 0;
                                }
                            }
                        }
                    }
                    self.rest.dma_channels[i] = d;
                })
            }
            while self.rest.total_apu_clocks * 1_000_000 / 1_024_000
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
        self.cpu.core.pbr as usize * 0x10000 + self.cpu.core.pc as usize
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
