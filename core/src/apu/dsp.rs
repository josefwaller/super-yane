use crate::{apu::Voice, utils::bit};
use log::*;
use std::collections::VecDeque;

/// The DSP
#[derive(Clone, Default)]
pub struct Dsp {
    /// The sound channels
    pub channels: [Voice; 8],
    /// Sample directory
    pub sample_dir: usize,
    /// The generated samples
    pub(super) sample_queue: VecDeque<f32>,
}

impl Dsp {
    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0x4C => {
                // debug!("Write");
                self.channels.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        // debug!("Start channel {i}");
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
                // debug!("Unknown DSP register {address:04X} value={value:02X}");
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
