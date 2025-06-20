use log::*;
use wdc65816::{HasAddressBus, Processor};

use crate::Cartridge;

pub struct Console {
    pub cpu: Processor,
    pub ram: [u8; 0x20000],
    pub cartridge: Cartridge,
}

macro_rules! wrapper {
    ($self: ident) => {{
        struct Wrapper<'a> {
            ram: &'a mut [u8; 0x20000],
            cartridge: &'a mut Cartridge,
        }
        impl HasAddressBus for Wrapper<'_> {
            fn io(&mut self) {}
            fn read(&self, address: usize) -> u8 {
                let a = address;
                let v = if (0x7E0000..0x800000).contains(&a) {
                    self.ram[a - 0x7E0000]
                } else if a % 0x800000 < 0x8000 {
                    let a = a & 0xFFFF;
                    if a < 0x2000 { self.ram[a] } else { 0 }
                } else {
                    self.cartridge.read_byte(a)
                };
                // debug!("Reading {:X} from {:X}", v, a);
                v
            }
            fn write(&mut self, address: usize, value: u8) {
                // debug!("Writing {:X} to {:X}", value, address);
                let a = address;
                if (0x7E0000..0x800000).contains(&a) {
                    self.ram[a - 0x7E0000] = value;
                } else if a % 0x800000 < 0x8000 {
                    let a = a & 0xFFFF;
                    if a < 0x2000 {
                        self.ram[a] = value;
                    } else if a < 0x2100 {
                        // Open bus?
                    } else if a < 0x2140 {
                        // PPU Registers
                        debug!("Wrote {:X} to PPU Register {:X}", value, a)
                    } else {
                    }
                } else {
                    // self.cartridge.read_byte(a)
                }
            }
        }
        Wrapper {
            ram: &mut $self.ram,
            cartridge: &mut $self.cartridge,
        }
    }};
}

impl Console {
    pub fn with_cartridge(cartridge_data: &[u8]) -> Console {
        let mut c = Console {
            cpu: Processor::default(),
            ram: [0; 0x20000],
            cartridge: Cartridge::from_data(cartridge_data),
        };
        c.cpu.pc = c.read_byte(0xFFFC) as u16 + 0x100 * c.read_byte(0xFFFD) as u16;
        debug!("Initialized PC to {:X}", c.cpu.pc);
        c
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        let a = address;
        if (0x7E0000..0x800000).contains(&a) {
            self.ram[a - 0x7E0000]
        } else if a % 0x800000 < 0x8000 {
            let a = a & 0xFFFF;
            if a < 0x2000 { self.ram[a] } else { 0 }
        } else {
            self.cartridge.read_byte(a)
        }
    }
    pub fn advance_instructions(&mut self, num_instructions: u32) {
        let mut wrapper = wrapper!(self);
        (0..num_instructions).for_each(|_| self.cpu.step(&mut wrapper))
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
