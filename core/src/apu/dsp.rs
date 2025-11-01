use crate::{
    apu::{
        Voice,
        voice::{AdsrStage, ENVELOPE_MAX_VALUE, GainMode},
    },
    utils::bit,
};
use derivative::Derivative;
use log::*;
use std::{collections::VecDeque, ops::Shr};

const GAUSS_TABLE: [i16; 0x200] = [
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
    0x000, 0x000, 0x000, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
    0x001, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x003, 0x003, 0x003, 0x003, 0x003,
    0x004, 0x004, 0x004, 0x004, 0x004, 0x005, 0x005, 0x005, 0x005, 0x006, 0x006, 0x006, 0x006,
    0x007, 0x007, 0x007, 0x008, 0x008, 0x008, 0x009, 0x009, 0x009, 0x00A, 0x00A, 0x00A, 0x00B,
    0x00B, 0x00B, 0x00C, 0x00C, 0x00D, 0x00D, 0x00E, 0x00E, 0x00F, 0x00F, 0x00F, 0x010, 0x010,
    0x011, 0x011, 0x012, 0x013, 0x013, 0x014, 0x014, 0x015, 0x015, 0x016, 0x017, 0x017, 0x018,
    0x018, 0x019, 0x01A, 0x01B, 0x01B, 0x01C, 0x01D, 0x01D, 0x01E, 0x01F, 0x020, 0x020, 0x021,
    0x022, 0x023, 0x024, 0x024, 0x025, 0x026, 0x027, 0x028, 0x029, 0x02A, 0x02B, 0x02C, 0x02D,
    0x02E, 0x02F, 0x030, 0x031, 0x032, 0x033, 0x034, 0x035, 0x036, 0x037, 0x038, 0x03A, 0x03B,
    0x03C, 0x03D, 0x03E, 0x040, 0x041, 0x042, 0x043, 0x045, 0x046, 0x047, 0x049, 0x04A, 0x04C,
    0x04D, 0x04E, 0x050, 0x051, 0x053, 0x054, 0x056, 0x057, 0x059, 0x05A, 0x05C, 0x05E, 0x05F,
    0x061, 0x063, 0x064, 0x066, 0x068, 0x06A, 0x06B, 0x06D, 0x06F, 0x071, 0x073, 0x075, 0x076,
    0x078, 0x07A, 0x07C, 0x07E, 0x080, 0x082, 0x084, 0x086, 0x089, 0x08B, 0x08D, 0x08F, 0x091,
    0x093, 0x096, 0x098, 0x09A, 0x09C, 0x09F, 0x0A1, 0x0A3, 0x0A6, 0x0A8, 0x0AB, 0x0AD, 0x0AF,
    0x0B2, 0x0B4, 0x0B7, 0x0BA, 0x0BC, 0x0BF, 0x0C1, 0x0C4, 0x0C7, 0x0C9, 0x0CC, 0x0CF, 0x0D2,
    0x0D4, 0x0D7, 0x0DA, 0x0DD, 0x0E0, 0x0E3, 0x0E6, 0x0E9, 0x0EC, 0x0EF, 0x0F2, 0x0F5, 0x0F8,
    0x0FB, 0x0FE, 0x101, 0x104, 0x107, 0x10B, 0x10E, 0x111, 0x114, 0x118, 0x11B, 0x11E, 0x122,
    0x125, 0x129, 0x12C, 0x130, 0x133, 0x137, 0x13A, 0x13E, 0x141, 0x145, 0x148, 0x14C, 0x150,
    0x153, 0x157, 0x15B, 0x15F, 0x162, 0x166, 0x16A, 0x16E, 0x172, 0x176, 0x17A, 0x17D, 0x181,
    0x185, 0x189, 0x18D, 0x191, 0x195, 0x19A, 0x19E, 0x1A2, 0x1A6, 0x1AA, 0x1AE, 0x1B2, 0x1B7,
    0x1BB, 0x1BF, 0x1C3, 0x1C8, 0x1CC, 0x1D0, 0x1D5, 0x1D9, 0x1DD, 0x1E2, 0x1E6, 0x1EB, 0x1EF,
    0x1F3, 0x1F8, 0x1FC, 0x201, 0x205, 0x20A, 0x20F, 0x213, 0x218, 0x21C, 0x221, 0x226, 0x22A,
    0x22F, 0x233, 0x238, 0x23D, 0x241, 0x246, 0x24B, 0x250, 0x254, 0x259, 0x25E, 0x263, 0x267,
    0x26C, 0x271, 0x276, 0x27B, 0x280, 0x284, 0x289, 0x28E, 0x293, 0x298, 0x29D, 0x2A2, 0x2A6,
    0x2AB, 0x2B0, 0x2B5, 0x2BA, 0x2BF, 0x2C4, 0x2C9, 0x2CE, 0x2D3, 0x2D8, 0x2DC, 0x2E1, 0x2E6,
    0x2EB, 0x2F0, 0x2F5, 0x2FA, 0x2FF, 0x304, 0x309, 0x30E, 0x313, 0x318, 0x31D, 0x322, 0x326,
    0x32B, 0x330, 0x335, 0x33A, 0x33F, 0x344, 0x349, 0x34E, 0x353, 0x357, 0x35C, 0x361, 0x366,
    0x36B, 0x370, 0x374, 0x379, 0x37E, 0x383, 0x388, 0x38C, 0x391, 0x396, 0x39B, 0x39F, 0x3A4,
    0x3A9, 0x3AD, 0x3B2, 0x3B7, 0x3BB, 0x3C0, 0x3C5, 0x3C9, 0x3CE, 0x3D2, 0x3D7, 0x3DC, 0x3E0,
    0x3E5, 0x3E9, 0x3ED, 0x3F2, 0x3F6, 0x3FB, 0x3FF, 0x403, 0x408, 0x40C, 0x410, 0x415, 0x419,
    0x41D, 0x421, 0x425, 0x42A, 0x42E, 0x432, 0x436, 0x43A, 0x43E, 0x442, 0x446, 0x44A, 0x44E,
    0x452, 0x455, 0x459, 0x45D, 0x461, 0x465, 0x468, 0x46C, 0x470, 0x473, 0x477, 0x47A, 0x47E,
    0x481, 0x485, 0x488, 0x48C, 0x48F, 0x492, 0x496, 0x499, 0x49C, 0x49F, 0x4A2, 0x4A6, 0x4A9,
    0x4AC, 0x4AF, 0x4B2, 0x4B5, 0x4B7, 0x4BA, 0x4BD, 0x4C0, 0x4C3, 0x4C5, 0x4C8, 0x4CB, 0x4CD,
    0x4D0, 0x4D2, 0x4D5, 0x4D7, 0x4D9, 0x4DC, 0x4DE, 0x4E0, 0x4E3, 0x4E5, 0x4E7, 0x4E9, 0x4EB,
    0x4ED, 0x4EF, 0x4F1, 0x4F3, 0x4F5, 0x4F6, 0x4F8, 0x4FA, 0x4FB, 0x4FD, 0x4FF, 0x500, 0x502,
    0x503, 0x504, 0x506, 0x507, 0x508, 0x50A, 0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511,
    0x511, 0x512, 0x513, 0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518,
    0x518, 0x518, 0x518, 0x519, 0x519,
];

/// The DSP
#[derive(Clone, Derivative)]
#[derivative(Default)]
pub struct Dsp {
    /// The sound channels
    pub channels: [Voice; 8],
    /// Sample directory
    pub sample_dir: usize,
    /// FIR (Finite Impulse Response) co-efficients
    pub fir_coeffs: [i8; 8],
    /// Previous 8 samples, used for the FIR/echo effect
    pub fir_cache: [[i16; 2]; 8],
    /// Size of the echo memory buffer
    pub echo_size: usize,
    /// Address of the echo buffer
    pub echo_addr: usize,
    /// Feedback, or volume of the echo to rewrite to RAM
    pub echo_feedback: i8,
    /// Echo volumes, left then right
    pub echo_volume: [i8; 2],
    /// Index of head of fir cache
    fir_index: usize,
    /// Index of the echo sample about to be read
    echo_index: usize,
    /// The generated samples
    pub(super) sample_queue: VecDeque<f32>,
}

impl Dsp {
    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0D => self.echo_feedback = value as i8,
            0x2C => self.echo_volume[0] = value as i8,
            0x3C => self.echo_volume[1] = value as i8,
            0x2D => {
                self.channels
                    .iter_mut()
                    .enumerate()
                    .skip(1)
                    .for_each(|(i, c)| c.pitch_mod_enabled = bit(value, i));
            }
            0x3D => {
                self.channels
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, c)| c.echo_enabled = bit(value, i));
            }
            0x4C => {
                self.channels.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.enabled = true;
                        // If we are not using ADSR, we don't want to reset the envelope
                        if c.adsr_enabled {
                            c.adsr_stage = AdsrStage::Attack;
                            c.envelope = 0;
                        }
                    }
                });
            }
            0x5C => {
                self.channels.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.adsr_stage = AdsrStage::Release;
                    }
                });
            }
            0x5D => {
                self.sample_dir = (value as usize) << 8;
            }
            0x6D => self.echo_addr = (value as usize) << 8,
            0x7D => self.echo_size = 512 * value as usize,
            0x6C => {}
            reg if address & 0x0F < 0x0A => {
                let channel_index = (reg / 0x10) & 0x0F;
                if channel_index < self.channels.len() {
                    let c = &mut self.channels[channel_index];
                    match reg & 0x0F {
                        0 => c.volume[0] = value as i8,
                        1 => c.volume[1] = value as i8,
                        2 => {
                            c.sample_pitch = (c.sample_pitch & 0x3F00) | (value as u16);
                        }
                        3 => {
                            c.sample_pitch =
                                ((value & 0x3F) as u16 * 0x100) | (c.sample_pitch & 0xFF);
                        }
                        4 => {
                            c.sample_src = (value as usize) * 0x04;
                        }
                        5 => {
                            c.adsr_enabled = bit(value, 7);
                            c.decay_rate = 0x2 * ((value >> 4) as usize & 0x07) + 0x10;
                            c.attack_rate = 0x2 * (value as usize & 0xF) + 1;
                            // c.envelope = 0;
                        }
                        6 => {
                            c.sustain_rate = (value & 0x1F) as usize;
                            c.sustain_level = ((value >> 5) as u32 + 1) * 0x100;
                        }
                        7 => {
                            if !bit(value, 7) {
                                c.envelope = (value as u16 & 0x7F) * 0x10;
                                c.gain_mode = GainMode::Fixed;
                            } else {
                                c.gain_rate = (value & 0x1F) as usize;
                                c.gain_mode = GainMode::from(value >> 5);
                            }
                        }
                        _ => {}
                    }
                }
            }
            v if address & 0x0F == 0x0F => {
                let index = v >> 4;
                if index < 8 {
                    self.fir_coeffs[index] = value as i8;
                }
            }
            _ => {
                // Ignore for now
                // debug!("Unknown DSP register {address:04X} value={value:02X}");
            }
        }
    }
    pub fn clock(&mut self, total_clocks: usize) {
        // self.channels.iter_mut().for_each(|c| c.clock(total_clocks));
    }
    pub fn read(&mut self, address: usize) -> u8 {
        debug!("APU read {address:04X}");
        0
    }
    pub fn generate_sample(&mut self, ram: &mut [u8]) {
        let mut prev_pitch: i32 = 0;

        let voices: [[i32; 2]; 8] = core::array::from_fn(|i| {
            let c = &mut self.channels[i];
            if c.enabled {
                c.clock(0);
                // Add sample pitch to counter
                let (counter, o) = c.counter.overflowing_add(if c.pitch_mod_enabled {
                    ((c.sample_pitch as i32 * ((prev_pitch >> 4) + 0x400)) >> 10)
                        .clamp(0, u16::MAX as i32) as u16
                } else {
                    c.sample_pitch
                });
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
                    // High nibble
                    let shift = head >> 4;
                    // Low nibble
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
                                // shift right by 1 since the sample is out of 15 bits
                                if shift >= 0xD {
                                    // When shift=13..15, decoding works as if shift=12 and nibble=(nibble SAR 3).
                                    (((v >> 3) << 12) as i16) >> 1
                                } else {
                                    ((v << shift) as i16) >> 1
                                }
                            })
                        });
                    c.prev_sample_data.copy_from_slice(&c.samples);

                    let mut old = c.samples[15] as i32;
                    let mut older = c.samples[14] as i32;
                    let samples: [i16; 16] = core::array::from_fn(|i| {
                        let s = sample_bytes.as_flattened()[i] as i32;
                        let value = match filter {
                            0 => s,
                            1 => s + old + ((-old) >> 4),
                            2 => s + 2 * old + ((-3 * old) >> 5) - older + (older >> 4),
                            3 => s + 2 * old + ((-13 * old) >> 6) - older + ((older * 3) >> 4),
                            _ => unreachable!(),
                        } as i16;
                        older = old;
                        old = value as i32;
                        value
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
            let gauss_index = (c.counter as usize & 0xFF0) >> 4;
            // Gaussian interpolation goes here
            let sample = (0..4)
                .map(|i| {
                    // i = 0 => newest, i = 3 => oldest
                    let gauss_value = match i {
                        0 => GAUSS_TABLE[0x000 + gauss_index],
                        1 => GAUSS_TABLE[0x100 + gauss_index],
                        2 => GAUSS_TABLE[0x1FF - gauss_index],
                        3 => GAUSS_TABLE[0x0FF - gauss_index],
                        _ => unreachable!(),
                    } as i32;
                    ((gauss_value
                        * if i > sample_index {
                            c.prev_sample_data[c.prev_sample_data.len() - (i - sample_index)]
                        } else {
                            c.samples[sample_index - i]
                        } as i32)
                        >> 10) as i32
                })
                .fold(0i32, |a, b| a.saturating_add(b))
                .shr(1i32)
                .clamp(i16::MIN as i32, i16::MAX as i32);

            prev_pitch = sample;

            let s = (sample * c.envelope as i32) / ENVELOPE_MAX_VALUE as i32;

            // [(s * c.volume[0] as i32) >> 7, (s * c.volume[1] as i32) >> 7]
            core::array::from_fn(|i| ((s * c.volume[i] as i32) >> 7) as i32)
        });

        let new_echo_value = voices
            .iter()
            .enumerate()
            .filter(|(i, _)| self.channels[*i].echo_enabled)
            .fold([0, 0], |acc, (_, v)| [acc[0] + v[0], acc[1] + v[1]]);

        // Read echo value
        if self.echo_size != 0 {
            // Read 4 bytes
            let values: [u8; 4] =
                core::array::from_fn(|i| ram[(self.echo_addr + self.echo_index + i) % ram.len()]);
            // Transform into 2 LE 16bit values
            self.fir_cache[self.fir_index] =
                core::array::from_fn(|i| i16::from_le_bytes([values[2 * i], values[2 * i + 1]]));
        }
        // Compute echo value
        let echo_val: i16 = (0..8)
            .map(|i| {
                // TODO: There shouldn't be 0 in this line
                ((self.fir_coeffs[i] as i32
                    * self.fir_cache[(self.fir_index + 7 - i) % 8][0] as i32)
                    >> 6) as i16
            })
            .sum();
        // Get the sample
        let echo_out: [i16; 2] = core::array::from_fn(|i| {
            (voices.iter().map(|arr| arr[i]).sum::<i32>() / self.channels.len() as i32
                + ((echo_val as i32 * self.echo_volume[i] as i32) >> 7)) as i16
        });
        let echo_in: [i16; 2] = core::array::from_fn(|i| {
            // (voices.iter().map(|arr| arr[i]).sum::<i32>() / self.channels.len() as i32
            //     + ((echo_val as i32 * self.echo_feedback as i32) >> 7)) as i16
            new_echo_value.map(|v| ((v * self.echo_feedback as i32) >> 7) as i16)[i]
        });

        // Go to next cache value
        self.fir_index = if self.fir_index == 0 {
            self.fir_cache.len() - 1
        } else {
            self.fir_index - 1
        };
        // Write value to memory
        if self.echo_size == 0 {
            self.fir_cache[self.fir_index] = echo_out;
        } else {
            [echo_in[0].to_le_bytes(), echo_in[1].to_le_bytes()]
                .as_flattened()
                .iter()
                .enumerate()
                .for_each(|(i, v)| ram[(self.echo_addr + self.echo_index + i) % ram.len()] = *v);
            self.echo_index = (self.echo_index + 4) % self.echo_size;
        }
        let s = (echo_out[0] + echo_out[1]) as f32 / 2.0 / 0x3FFF as f32;
        if s > 1.0 {
            error!("Invalid audio sample generated: {}", s);
            self.sample_queue.push_back(0.0);
        } else {
            self.sample_queue.push_back(s);
        }
    }
}
