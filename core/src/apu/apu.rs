use derive_new::new;
use std::collections::VecDeque;

use crate::{apu::Dsp, utils::bit};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_big_array::Array;
use spc700::{HasAddressBus, IPL, Processor as Spc700Processor};

use derivative::Derivative;
use paste::paste;

pub const APU_RAM_SIZE: usize = 0x10000;
/// Generate a new sample every 64 APU clocks (32 SPC700 clocks)
pub const CLOCKS_PER_SAMPLE: usize = 96;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, new)]
pub struct ApuTimer {
    #[new(value = "false")]
    pub enabled: bool,
    // Set by program
    #[new(value = "0")]
    pub timer_target: u8,
    // Automatically counts up
    #[new(value = "0")]
    pub timer_value: u8,
    /// Really u4
    #[new(value = "0")]
    pub counter: u8,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Apu {
    pub core: Spc700Processor,
    pub rest: ApuMemory,
}

// Internal struct used to advance the APU core
#[derive(Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
pub struct ApuMemory {
    #[derivative(Default(value = "Box::new(Array([0; APU_RAM_SIZE]))"))]
    pub ram: Box<Array<u8, APU_RAM_SIZE>>,
    pub cpu_to_apu_reg: [u8; 4],
    pub apu_to_cpu_reg: [u8; 4],
    pub timers: [ApuTimer; 3],
    /// Total number of clocks that have passed
    pub total_clocks: usize,
    #[derivative(Default(value = "true"))]
    pub expose_ipl_rom: bool,
    pub dsp_addr: usize,
    pub dsp: Dsp,
    pub dsp_read_only: bool,
}

impl ApuMemory {
    pub fn advance_apu_clocks(&mut self, clocks: usize) {
        // Advance timers
        (0..clocks).for_each(|_| {
            self.total_clocks += 1;
            if self.total_clocks % 3 == 0 {
                // Clock the timers every 16 (timers 0 and 1) or 128 (timer 2) APU cycles
                [128, 128, 16]
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, clks)| {
                        if (self.total_clocks / 3) % clks == 0 {
                            let t = &mut self.timers[i];
                            // Increment timer and increment counter if it overflows
                            t.timer_value = t.timer_value.wrapping_add(1);
                            if t.timer_value == t.timer_target {
                                t.counter = t.counter.wrapping_add(1);
                                t.timer_value = 0;
                            }
                        }
                    });
                if self.total_clocks % CLOCKS_PER_SAMPLE == 0 {
                    self.dsp.generate_sample(self.ram.as_mut_slice());
                }
            }
        });
    }
}

impl HasAddressBus for ApuMemory {
    fn io(&mut self) {
        // Advance by 2 cycles since the SPC700 is only clocked on every other clock
        self.advance_apu_clocks(2);
    }
    fn read(&mut self, address: usize) -> u8 {
        self.advance_apu_clocks(2);
        match address {
            0x00F2 => self.dsp_addr as u8,
            0x00F3 => self.dsp.read(self.dsp_addr),
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
        self.advance_apu_clocks(2);
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
                self.timers[0].enabled = value & 0x01 != 0;
                self.timers[1].enabled = value & 0x02 != 0;
                self.timers[2].enabled = value & 0x04 != 0;
            }
            0x00F2 => {
                self.dsp_addr = (value & 0x7F) as usize;
                self.dsp_read_only = bit(value, 7);
            }
            0x00F3 => {
                if !self.dsp_read_only {
                    self.dsp.write(self.dsp_addr, value);
                }
            }
            0x00F4..0x00F8 => self.apu_to_cpu_reg[address - 0x00F4] = value,
            0x00FA..0x00FD => {
                self.timers[address - 0x00FA].timer_target = value;
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
    pub fn sample_queue(&mut self) -> VecDeque<f32> {
        let mut s = VecDeque::new();
        std::mem::swap(&mut self.rest.dsp.sample_queue, &mut s);
        s
    }

    pub fn reset(&mut self) {
        self.core.reset();
        self.rest.expose_ipl_rom = true;
        self.rest.timers.iter_mut().for_each(|i| i.counter = 0);
        // Silence every channel
        self.rest
            .dsp
            .channels
            .iter_mut()
            .for_each(|c| c.enabled = false);
    }
}
