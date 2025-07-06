use log::*;
use wdc65816::{HasAddressBus, Processor};

use crate::{Cartridge, Ppu};
use paste::paste;

#[derive(Copy, Clone, Default, Debug)]
pub enum DmaAddressAjustMode {
    #[default]
    Increment,
    Decrement,
    Fixed,
}

#[derive(Clone)]
pub struct DmaChannel {
    pub transfer_pattern: Vec<u32>,
    pub adjust_mode: DmaAddressAjustMode,
    pub indirect: bool,
    pub direction: bool,
    pub dest_addr: usize,
    /// The lower 16 bits of the DMA address
    pub src_addr: u16,
    /// The bank of the DMA address
    pub src_bank: u8,
    /// The byte counter, or number of bytes to transfer
    pub byte_counter: u16,
}

impl Default for DmaChannel {
    fn default() -> Self {
        DmaChannel {
            transfer_pattern: vec![0],
            adjust_mode: DmaAddressAjustMode::Increment,
            indirect: false,
            direction: false,
            dest_addr: 0,
            src_addr: 0,
            src_bank: 0,
            byte_counter: 0,
        }
    }
}
#[derive(Clone)]
pub struct ExternalArchitecture {
    pub ram: [u8; 0x20000],
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    /// DMA Channels
    pub dma_channels: [DmaChannel; 8],
    total_master_clocks: u32,
    open_bus_value: u8,
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
                if a < 0x2000 {
                    self.ram[a] = value;
                    12
                } else if a < 0x2100 {
                    // Open bus?
                    12
                } else if a < 0x2140 {
                    // PPU Registers
                    self.ppu.write_byte(a, value);
                    12
                } else if a < 0x4400 {
                    if a == 0x420B {
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
                                    let v = self.read(src);
                                    self.write(dest, v);
                                    let md = &mut self.dma_channels[i];
                                    bytes_transferred += 1;
                                    match md.adjust_mode {
                                        DmaAddressAjustMode::Increment => {
                                            md.src_addr = d.src_addr.wrapping_add(1)
                                        }
                                        DmaAddressAjustMode::Decrement => {
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
                    if a >= 0x4300 {
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
                                    0x00 => DmaAddressAjustMode::Increment,
                                    0x10 => DmaAddressAjustMode::Decrement,
                                    _ => DmaAddressAjustMode::Fixed,
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
                    } else {
                        12
                    }
                } else {
                    12
                }
            } else {
                // self.cartridge.read_byte(a)
                12
            }
        }
    }
    fn advance(&mut self, master_clocks: u32) {
        self.total_master_clocks += master_clocks;
        self.ppu.advance_master_clock(master_clocks);
    }
}
impl HasAddressBus for ExternalArchitecture {
    fn io(&mut self) {
        // self.advance(6);
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

/// The entire S.N.E.S. Console
#[derive(Clone)]
pub struct Console {
    // The processor is the driving force behind the emulator, so the console is split into 2 parts
    // so that the CPU can be passed the rest of the console, and advance it in sync
    // i.e. cpu.advance_instructions(&rest) will advacne both by 3 CPU instructions
    // Since this is a bit of a weird structure we provide methods to get the different parts of the console as mutable/immuatable
    // references
    cpu: Processor,
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
    rest_field! {total_master_clocks, u32}
    pub fn cpu(&self) -> &Processor {
        &self.cpu
    }
    pub fn cpu_mut(&mut self) -> &mut Processor {
        &mut self.cpu
    }
    pub fn with_cartridge(cartridge_data: &[u8]) -> Console {
        let mut c = Console {
            cpu: Processor::default(),
            rest: ExternalArchitecture {
                ram: [0; 0x20000],
                cartridge: Cartridge::from_data(cartridge_data),
                ppu: Ppu::default(),
                dma_channels: core::array::from_fn(|_| DmaChannel::default()),
                total_master_clocks: 0,
                open_bus_value: 0,
            },
        };
        c.cpu.pc =
            c.cartridge().read_byte(0xFFFC) as u16 + 0x100 * c.cartridge().read_byte(0xFFFD) as u16;
        debug!("Initialized PC to {:X}", c.cpu.pc);
        c
    }
    pub fn advance_instructions(&mut self, num_instructions: u32) {
        (0..num_instructions).for_each(|_| {
            self.cpu.step(&mut self.rest);
        });
    }
    pub fn advance_until(&mut self, should_stop: &mut impl FnMut(&Console) -> bool) -> u32 {
        std::iter::from_fn(|| {
            if should_stop(&self) {
                None
            } else {
                self.cpu.step(&mut self.rest);
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
    pub fn read_byte(&self, address: usize) -> u8 {
        self.rest.read_byte(address).0
    }
}
