use log::*;
use wdc65816::{HasAddressBus, Processor};

use crate::{Cartridge, Ppu};

#[derive(Copy, Clone, Default, Debug)]
pub enum DmaAddressAjustMode {
    #[default]
    Increment,
    Decrement,
    Fixed,
}

#[derive(Clone, Default)]
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

pub struct Console {
    pub cpu: Processor,
    pub ram: [u8; 0x20000],
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    /// DMA Channels
    pub dma_channels: [DmaChannel; 8],
}

macro_rules! wrapper {
    ($self: ident) => {{
        struct Wrapper<'a> {
            ram: &'a mut [u8; 0x20000],
            cartridge: &'a mut Cartridge,
            ppu: &'a mut Ppu,
            dma_channels: &'a mut [DmaChannel; 8],
        }
        impl HasAddressBus for Wrapper<'_> {
            fn io(&mut self) {}
            // Todo: Add more than just LoRom support for these
            fn read(&mut self, address: usize) -> u8 {
                let a = address;
                let v = if (0x7E0000..0x800000).contains(&a) {
                    self.ram[a - 0x7E0000]
                } else if a % 0x800000 < 0x8000 {
                    let a = a & 0xFFFF;
                    if a < 0x2000 {
                        self.ram[a]
                    } else if a < 0x2100 {
                        0
                    } else if a < 0x2140 {
                        self.ppu.read_byte(a)
                    } else {
                        self.ppu.read_byte(a)
                    }
                } else {
                    self.cartridge.read_byte(a)
                };
                v
            }
            fn write(&mut self, address: usize, value: u8) {
                let a = address;
                // Check for RAM
                if (0x7E0000..0x800000).contains(&a) {
                    self.ram[a - 0x7E0000] = value;
                } else {
                    let a = a % (0x800000);
                    // Check for non-rom area
                    if a < 0x400000 && a & 0xFFFF < 0x8000 {
                        let a = a % 0x8000;
                        if a < 0x2000 {
                            self.ram[a] = value;
                        } else if a < 0x2100 {
                            // Open bus?
                        } else if a < 0x2140 {
                            // PPU Registers
                            self.ppu.write_byte(address, value);
                        } else if a < 0x4400 {
                            if a == 0x420B {
                                (0..8).for_each(|i| {
                                    if (a >> i) & 0x01 != 0 {
                                        let d = &mut self.dma_channels[i].clone();
                                        let mut i = 0;
                                        // Todo: handling timing of DMA
                                        while d.byte_counter > 0 {
                                            let src =
                                                d.src_bank as usize * 0x1000 + d.src_addr as usize;
                                            let dest = d.dest_addr
                                                + d.transfer_pattern[i % d.transfer_pattern.len()]
                                                    as usize;
                                            i += 1;
                                            let v = self.read(src);
                                            self.ppu.write_byte(dest, v);
                                            match d.adjust_mode {
                                                DmaAddressAjustMode::Increment => {
                                                    d.src_addr = d.src_addr.wrapping_add(1)
                                                }
                                                DmaAddressAjustMode::Decrement => {
                                                    d.src_addr = d.src_addr.wrapping_sub(1)
                                                }
                                                _ => {}
                                            }
                                            d.byte_counter -= 1;
                                        }
                                    }
                                });
                            }
                            if a >= 0x4300 {
                                let lsb = a & 0x0F;
                                let r = (a & 0xF0) >> 4;
                                if r > 7 {
                                    return;
                                }
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
                                    d.byte_counter =
                                        (d.byte_counter & 0x00FF) | (value as u16 * 0x100);
                                }
                            }
                        }
                    } else {
                        // self.cartridge.read_byte(a)
                    }
                }
            }
        }
        Wrapper {
            ram: &mut $self.ram,
            cartridge: &mut $self.cartridge,
            ppu: &mut $self.ppu,
            dma_channels: &mut $self.dma_channels,
        }
    }};
}

impl Console {
    pub fn with_cartridge(cartridge_data: &[u8]) -> Console {
        let mut c = Console {
            cpu: Processor::default(),
            ram: [0; 0x20000],
            cartridge: Cartridge::from_data(cartridge_data),
            ppu: Ppu::default(),
            dma_channels: core::array::from_fn(|_| DmaChannel::default()),
        };
        c.cpu.pc =
            c.cartridge.read_byte(0xFFFC) as u16 + 0x100 * c.cartridge.read_byte(0xFFFD) as u16;
        debug!("Initialized PC to {:X}", c.cpu.pc);
        c
    }
    pub fn advance_instructions(&mut self, num_instructions: u32) {
        let mut wrapper = wrapper!(self);
        (0..num_instructions).for_each(|_| {
            self.cpu.step(&mut wrapper);
        });
        wrapper.ppu.advance_master_clock(1);
    }
    pub fn advance_until(&mut self, should_stop: &mut impl FnMut(&Console) -> bool) -> u32 {
        std::iter::from_fn(|| {
            if should_stop(&self) {
                None
            } else {
                let mut wrapper = wrapper!(self);
                self.cpu.step(&mut wrapper);
                Some(1)
            }
        })
        .sum()
    }
}
