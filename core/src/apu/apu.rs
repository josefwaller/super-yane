use log::debug;
use spc700::{HasAddressBus, IPL, Processor as Spc700Processor};

use derivative::Derivative;
use paste::paste;

pub const APU_RAM_SIZE: usize = 0x10000;

#[derive(Debug, Clone, Copy, Default)]
pub struct ApuTimer {
    pub timer: u8,
    /// Really u4
    pub counter: u8,
}

#[derive(Clone, Default)]
pub struct Apu {
    pub core: Spc700Processor,
    rest: ApuMemory,
}

// Internal struct used to advance the APU core
#[derive(Clone, Derivative)]
#[derivative(Default)]
struct ApuMemory {
    #[derivative(Default(value = "Box::new([0; APU_RAM_SIZE])"))]
    pub ram: Box<[u8; APU_RAM_SIZE]>,
    pub cpu_to_apu_reg: [u8; 4],
    pub apu_to_cpu_reg: [u8; 4],
    pub timers: [ApuTimer; 3],
    /// Total number of clocks that have passed
    pub total_clocks: usize,
    #[derivative(Default(value = "true"))]
    pub expose_ipl_rom: bool,
}

impl ApuMemory {
    pub fn advance_clocks(&mut self, clocks: usize) {
        // Advance timers
        (0..clocks).for_each(|_| {
            self.total_clocks += 1;
            // Clock the timers every 16 (timers 0 and 1) or 128 (timer 2) APU cycles
            [16, 16, 128].into_iter().enumerate().for_each(|(i, clks)| {
                if self.total_clocks % clks == 0 {
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
}

impl HasAddressBus for ApuMemory {
    fn io(&mut self) {
        self.advance_clocks(1);
    }
    fn read(&mut self, address: usize) -> u8 {
        self.advance_clocks(1);
        match address {
            0x00F4..0x00F8 => self.cpu_to_apu_reg[address - 0x00F4],
            0x00FD..0x00FF => {
                let v = self.timers[address - 0x00FD].counter;
                self.timers[address - 0x00FD].counter = 0;
                v
            }
            0x0000..0xFFC0 => self.ram[address],
            0xFFC0..0x10000 => {
                if self.expose_ipl_rom {
                    IPL[address - 0xFFC0]
                } else {
                    self.ram[address]
                }
            }
            _ => panic!("Should be impossible"),
        }
    }
    fn write(&mut self, address: usize, value: u8) {
        self.advance_clocks(1);
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
            _ => self.ram[address] = value,
        }
    }
}

// Expose a field in the `rest` struct via a method
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
impl Apu {
    rest_field! {total_clocks, usize}
    /// Takes the CPU to APU side registers and returns the values written to the APU to CPU registers
    pub fn step(&mut self, cpu_reg: [u8; 4]) -> [u8; 4] {
        self.rest.cpu_to_apu_reg = cpu_reg;
        self.core.step(&mut self.rest);
        self.rest.apu_to_cpu_reg
    }
    /// Reads from ram, and thus doesn't require a mutable reference
    pub fn read_ram(&self, address: usize) -> u8 {
        self.rest.ram[address % APU_RAM_SIZE]
    }
}
