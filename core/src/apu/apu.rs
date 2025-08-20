use std::collections::VecDeque;

use crate::{apu::Voice, utils::bit};
use log::{debug, error};
use spc700::{HasAddressBus, IPL, Processor as Spc700Processor};

use derivative::Derivative;
use paste::paste;

pub const APU_RAM_SIZE: usize = 0x10000;
/// Generate a new sample every 32 clocks
pub const CLOCKS_PER_SAMPLE: usize = 32;

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

/// The DSP
#[derive(Clone, Default)]
pub struct Dsp {
    /// The sound channels
    pub channels: [Voice; 8],
    /// Sample directory
    pub sample_dir: usize,
    /// The generated samples
    sample_queue: VecDeque<f32>,
}

impl Dsp {
    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0x4C => {
                self.channels.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.enabled = true;
                    }
                });
            }
            0x5C => {
                self.channels.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.enabled = false
                    }
                });
            }
            0x5D => {
                self.sample_dir = (value as usize) << 8;
            }
            0x6C => {}
            reg if address & 0x0F < 0x0A => {
                let channel_index = (reg / 0x10) & 0x0F;
                if channel_index < self.channels.len() {
                    let c = &mut self.channels[channel_index];
                    match reg & 0x0F {
                        0 => c.volume[0] = value,
                        1 => c.volume[1] = value,
                        2 => {
                            c.sample_pitch = (c.sample_pitch & 0x3F00) | (value as u16);
                            // debug!("Sample rate is {:04X}", c.sample_pitch);
                        }
                        3 => {
                            c.sample_pitch =
                                ((value & 0x3F) as u16 * 0x100) | (c.sample_pitch & 0xFF);
                            // debug!("Sample rate is {:04X}", c.sample_pitch);
                        }
                        4 => {
                            c.sample_src = (value as usize) * 0x04;
                        }
                        // todo
                        5..=9 => {}
                        _ => unreachable!(),
                    }
                }
            }
            _ => {
                // Ignore for now
                debug!("Unknown DSP register {address:04X} value={value:02X}");
            }
        }
    }
    pub fn read(&mut self, address: usize) -> u8 {
        debug!("APU read {address:04X}");
        0
    }
    pub fn generate_sample(&mut self, ram: &[u8]) {
        let s = self
            .channels
            .iter_mut()
            .map(|c| {
                if c.enabled {
                    // Add sample pitch to counter
                    let (counter, o) = c.counter.overflowing_add(c.sample_pitch);
                    // If overflowed, we need to load the next block
                    if o {
                        // If enabled and block address is None, we need to load the first address
                        let block_addr = c.block_addr.unwrap_or_else(|| {
                            // Get the directory address
                            let addr = self.sample_dir + c.sample_src;
                            // debug!("Reading from APU ram {addr:04X}");
                            u16::from_le_bytes([ram[addr], ram[addr + 1]]) as usize
                        });
                        // Read the head and parse into flags
                        let head = ram[block_addr];
                        let shift = head >> 4;
                        let filter = (head >> 2) & 0x03;
                        let loop_flag = bit(head, 1);
                        let end_flag = bit(head, 0);
                        // Read and parse blocks
                        let sample_bytes: [[i16; 2]; 8] =
                            core::array::from_fn(|i| ram[block_addr + 1 + i]).map(|v| {
                                [v >> 4, v & 0xF].map(|v| {
                                    // If negative, flip all the other bits
                                    let v = if v > 0x07 {
                                        0xFFF0 | (v as u16)
                                    } else {
                                        v as u16
                                    };
                                    (v << shift) as i16
                                })
                            });
                        let samples: [i16; 16] = core::array::from_fn(|i| {
                            let s = sample_bytes.as_flattened()[i] as i32;
                            let ps: [i32; 2] = core::array::from_fn(|i| c.prev_samples[i] as i32);
                            c.prev_samples[1] = c.prev_samples[0];
                            c.prev_samples[0] = s as i16;
                            match filter {
                                0 => s,
                                1 => s + 15 * ps[0] / 16,
                                2 => s + 61 * ps[0] / 32 + 15 * ps[1] / 16,
                                3 => s + 115 * ps[0] / 64 + 13 * ps[1] / 16,
                                _ => unreachable!(),
                            }
                            .clamp(i16::MIN as i32, i16::MAX as i32)
                                as i16
                        });
                        c.samples.copy_from_slice(&samples);

                        // Get next block address
                        c.block_addr = if end_flag {
                            if loop_flag {
                                // Point to loop address
                                let addr = self.sample_dir + c.sample_src + 2;
                                Some(u16::from_le_bytes([ram[addr], ram[addr + 1]]) as usize)
                            } else {
                                // Disable channel
                                c.enabled = false;
                                None
                            }
                        } else {
                            Some(block_addr + 9)
                        };
                    }
                    c.counter = counter;
                }
                // Select the top 4 bits as the sample index
                let sample_index = (c.counter >> 12) as usize;
                // Gaussian interpolation goes here

                // For now, just return sample
                c.samples[sample_index] as f32 / std::i16::MAX as f32
            })
            .sum::<f32>()
            / self.channels.len() as f32;
        if s > 1.0 {
            error!("Invalid audio sample generated: {}", s);
            self.sample_queue.push_back(0.0);
        } else {
            // debug!("Add sample {}", s);
            self.sample_queue.push_back(s);
        }
    }
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
    pub dsp_addr: usize,
    pub dsp: Dsp,
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
            if self.total_clocks % CLOCKS_PER_SAMPLE == 0 {
                self.dsp.generate_sample(self.ram.as_slice());
            }
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
            0x00F2 => self.dsp_addr = (value & 0x7F) as usize,
            0x00F3 => self.dsp.write(self.dsp_addr, value),
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
    pub fn sample_queue(&mut self) -> VecDeque<f32> {
        let mut s = VecDeque::new();
        std::mem::swap(&mut self.rest.dsp.sample_queue, &mut s);
        s
    }
}
