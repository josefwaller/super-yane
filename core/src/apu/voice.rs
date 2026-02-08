use std::{collections::VecDeque, ops::Shr};

use crate::apu::constants::{
    ENVELOPE_MAX_VALUE, GAUSS_TABLE, LEFT, PERIOD_OFFSET_TABLE, PERIOD_TABLE, RELEASE_PERIOD_RATE,
    RIGHT,
};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::utils::bit;

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub enum GainMode {
    #[default]
    Fixed,
    LinearDecrease,
    ExponentialDecrease,
    LinearIncrease,
    BentIncrease,
}

impl From<u8> for GainMode {
    fn from(value: u8) -> Self {
        use GainMode::*;
        match value & 0x07 {
            0b100 => LinearDecrease,
            0b101 => ExponentialDecrease,
            0b110 => LinearIncrease,
            0b111 => BentIncrease,
            _ => Fixed,
        }
    }
}

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub enum AdsrStage {
    #[default]
    Attack,
    Decay,
    Sustain,
    Release,
}

impl ToString for AdsrStage {
    fn to_string(&self) -> String {
        use AdsrStage::*;
        match self {
            Attack => "Attack",
            Decay => "Decay",
            Sustain => "Sustain",
            Release => "Release",
        }
        .to_string()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub enabled: bool,
    /// Volume, first left, then right
    pub volume: [i8; 2],
    /// Sample pitch, i.e. rate at which samples are consumed
    pub sample_pitch: u16,
    /// Sample source
    pub sample_src: usize,
    /// ADSR enable
    pub adsr_enabled: bool,
    pub adsr_stage: AdsrStage,
    pub decay_rate: usize,
    pub attack_rate: usize,
    pub sustain_level: u32,
    pub sustain_rate: usize,
    pub gain_rate: usize,
    pub gain_mode: GainMode,
    pub echo_enabled: bool,
    /// End of block flag
    pub end_flag: bool,
    /// Current address of the block being played
    pub(super) block_addr: Option<usize>,
    /// Decoded samples from the BRR block
    pub(super) samples: [i16; 16],
    /// Copy of previous sample data, for gaussian interpolation
    pub(super) prev_sample_data: [i16; 16],
    /// Counter value
    pub(super) counter: u16,
    /// Whether pitch modulation is enabled
    pub pitch_mod_enabled: bool,
    /// The current envelope of the voic
    pub(super) envelope: u16,
    /// Internal period counter value
    period_counter: usize,
    /// Replace the output of the voice with noise
    pub noise_enabled: bool,
}

impl Voice {
    pub fn write(&mut self, addr: usize, value: u8) {
        match addr & 0x0F {
            0 => self.volume[LEFT] = value as i8,
            1 => self.volume[RIGHT] = value as i8,
            2 => {
                self.sample_pitch = (self.sample_pitch & 0x3F00) | (value as u16);
            }
            3 => {
                self.sample_pitch = ((value & 0x3F) as u16 * 0x100) | (self.sample_pitch & 0xFF);
            }
            4 => {
                self.sample_src = (value as usize) * 0x04;
            }
            5 => {
                self.adsr_enabled = bit(value, 7);
                self.decay_rate = 0x2 * ((value >> 4) as usize & 0x07) + 0x10;
                self.attack_rate = 0x2 * (value as usize & 0xF) + 1;
            }
            6 => {
                self.sustain_rate = (value & 0x1F) as usize;
                self.sustain_level = ((value >> 5) as u32 + 1) * 0x200;
            }
            7 => {
                if !bit(value, 7) {
                    self.envelope = (value as u16 & 0x7F) * 0x10;
                    self.gain_mode = GainMode::Fixed;
                } else {
                    self.gain_rate = (value & 0x1F) as usize;
                    self.gain_mode = GainMode::from(value >> 5);
                }
            }
            _ => {}
        }
    }
    pub fn read(&self, addr: usize) -> u8 {
        match addr & 0xF0 {
            0 => self.volume[LEFT] as u8,
            1 => self.volume[RIGHT] as u8,
            2 => self.sample_pitch.to_le_bytes()[0],
            3 => self.sample_pitch.to_le_bytes()[1],
            4 => (self.sample_src / 0x04) as u8,
            _ => {
                debug!("Read from {:02X}", addr);
                0
            }
        }
    }
    pub fn clock(&mut self) {
        self.period_counter = if self.period_counter == 0 {
            0x77FF
        } else {
            self.period_counter - 1
        };
        // Compute envelope value
        if self.adsr_enabled {
            use AdsrStage::*;
            self.envelope = match self.adsr_stage {
                Attack => {
                    let v = if self.get_period_elapsed(self.attack_rate) {
                        self.envelope + if self.attack_rate == 0x1F { 1024 } else { 32 }
                    } else {
                        self.envelope
                    };
                    if v >= 0x7E0 {
                        self.adsr_stage = Decay;
                    }
                    v.min(0x7FF)
                }
                Decay => {
                    if self.get_period_elapsed(self.decay_rate) {
                        let v = self.envelope.saturating_sub(1);
                        let v = v.saturating_sub((v >> 8) + 1);
                        if v & 0xE0 == (self.sustain_level & 0xE0) as u16 {
                            self.adsr_stage = Sustain;
                        }
                        v
                    } else {
                        self.envelope
                    }
                }
                Sustain => {
                    if self.get_period_elapsed(self.sustain_rate) {
                        let v = self.envelope.saturating_sub(1);
                        let v = v.saturating_sub((v >> 8) + 1);
                        v
                    } else {
                        self.envelope
                    }
                }
                Release => {
                    if self.get_period_elapsed(RELEASE_PERIOD_RATE) {
                        self.envelope.saturating_sub(8)
                    } else {
                        self.envelope
                    }
                }
            }
        } else {
            if self.get_period_elapsed(self.gain_rate) {
                use GainMode::*;
                self.envelope = match self.gain_mode {
                    Fixed => self.envelope,
                    LinearDecrease => self.envelope.saturating_sub(32),
                    ExponentialDecrease => self
                        .envelope
                        .saturating_sub(((self.envelope.saturating_sub(1)) >> 8) + 1),
                    LinearIncrease => self.envelope + 32,
                    BentIncrease => {
                        if self.envelope < 0x600 {
                            self.envelope + 32
                        } else {
                            self.envelope + 8
                        }
                    }
                }
                .clamp(0, ENVELOPE_MAX_VALUE);
            }
        }
        self.envelope = self.envelope.clamp(0, ENVELOPE_MAX_VALUE);
    }
    /// Generate a new sample from this voice
    /// Returned as a [right, left] array
    pub fn generate_sample(
        &mut self,
        sample_dir_addr: usize,
        prev_pitch: &mut i32,
        ram: &[u8],
        noise_value: i32,
    ) -> [i32; 2] {
        if self.enabled {
            self.clock();
            // Add sample pitch to counter
            let (counter, o) = self.counter.overflowing_add(if self.pitch_mod_enabled {
                ((self.sample_pitch as i32 * ((*prev_pitch >> 4) + 0x400)) >> 10)
                    .clamp(0, u16::MAX as i32) as u16
            } else {
                self.sample_pitch
            });
            // If overflowed, we need to load the next block
            if o {
                // If enabled and block address is None, we need to load the first address
                let block_addr = self.block_addr.unwrap_or_else(|| {
                    // Get the directory address
                    let addr = sample_dir_addr + self.sample_src;
                    u16::from_le_bytes([ram[addr], ram[addr + 1]]) as usize
                });
                // Read the head and parse into flags
                let head = ram[block_addr];
                // High nibble
                let shift = head >> 4;
                // Low nibble
                let filter = (head >> 2) & 0x03;
                let loop_flag = bit(head, 1);
                self.end_flag = bit(head, 0);
                // Read and parse blocks
                let sample_bytes: [[i16; 2]; 8] =
                    core::array::from_fn(|i| ram[(block_addr + 1 + i) % ram.len()]).map(|v| {
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
                self.prev_sample_data.copy_from_slice(&self.samples);

                let mut old = self.samples[15] as i32;
                let mut older = self.samples[14] as i32;
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
                self.samples.copy_from_slice(&samples);

                // Get next block address
                self.block_addr = if self.end_flag {
                    if loop_flag {
                        // Point to loop address
                        let addr = sample_dir_addr + self.sample_src + 2;
                        Some(u16::from_le_bytes([ram[addr], ram[addr + 1]]) as usize)
                    } else {
                        // Disable channel
                        self.enabled = false;
                        None
                    }
                } else {
                    Some(block_addr + 9)
                };
            }
            self.counter = counter;
        }
        // Select the top 4 bits as the sample index
        let sample_index = (self.counter >> 12) as usize;
        let gauss_index = (self.counter as usize & 0xFF0) >> 4;
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
                        self.prev_sample_data[self.prev_sample_data.len() - (i - sample_index)]
                    } else {
                        self.samples[sample_index - i]
                    } as i32)
                    >> 10) as i32
            })
            .fold(0i32, |a, b| a.saturating_add(b))
            .shr(1i32)
            .clamp(i16::MIN as i32, i16::MAX as i32);

        *prev_pitch = sample;

        let s = (if self.noise_enabled {
            noise_value
        } else {
            sample
        } as f32
            * self.envelope as f32
            / ENVELOPE_MAX_VALUE as f32)
            .floor() as i32;

        core::array::from_fn(|i| ((s * self.volume[i] as i32) >> 7) as i32)
    }
    fn get_period_elapsed(&self, rate: usize) -> bool {
        let table_val = PERIOD_TABLE[rate];
        table_val != 0 && (self.period_counter + PERIOD_OFFSET_TABLE[rate]) % table_val == 0
    }
}
