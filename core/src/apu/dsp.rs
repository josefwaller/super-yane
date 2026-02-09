use crate::{
    apu::{
        Voice,
        constants::{LEFT, PERIOD_TABLE, RIGHT},
        voice::AdsrStage,
    },
    utils::bit,
};
use derivative::Derivative;
use log::*;
use seeded_random::{Random, Seed};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The DSP
#[derive(Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
pub struct Dsp {
    /// The volume
    pub volume: [i8; 2],
    /// The voices, or sound channels
    pub voices: [Voice; 8],
    /// Sample directory
    pub sample_dir: usize,
    /// FIR (Finite Impulse Response) co-efficients
    pub fir_coeffs: [i8; 8],
    /// Previous 8 samples, used for the FIR/echo effect
    /// Implemented as a ring buffer
    pub fir_cache: [[i16; 2]; 8],
    /// Size of the echo memory buffer
    pub echo_size: usize,
    /// Address of the echo buffer
    pub echo_addr: usize,
    /// Feedback, or volume of the echo to rewrite to RAM
    pub echo_feedback: i8,
    /// Echo volumes, left then right
    pub echo_volume: [i8; 2],
    /// Noise frequency
    pub noise_frequency: usize,
    /// Noise index
    noise_index: usize,
    /// Index of head of fir cache
    fir_index: usize,
    /// Index of the echo sample about to be read
    echo_index: usize,
    /// The generated samples
    #[serde(skip)]
    pub(super) sample_queue: VecDeque<f32>,

    pub echo_enabled: bool,
}

impl Dsp {
    pub fn write(&mut self, address: usize, value: u8) {
        match address % 0x80 {
            0x0C => self.volume[LEFT] = value as i8,
            0x1C => self.volume[RIGHT] = value as i8,
            0x0D => self.echo_feedback = value as i8,
            0x2C => self.echo_volume[0] = value as i8,
            0x2D => {
                self.voices
                    .iter_mut()
                    .enumerate()
                    .skip(1)
                    .for_each(|(i, c)| c.pitch_mod_enabled = bit(value, i));
            }
            0x3C => self.echo_volume[1] = value as i8,
            0x3D => (0..8).for_each(|i| self.voices[i].noise_enabled = bit(value, i)),
            0x4D => {
                self.voices
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, c)| c.echo_enabled = bit(value, i));
            }
            0x4C => {
                self.voices.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.enabled = true;
                        c.adsr_stage = AdsrStage::Attack;
                        c.block_addr = None;
                        // If we are not using ADSR, we don't want to reset the envelope
                        if c.adsr_enabled {
                            c.envelope = 0;
                        }
                    }
                });
            }
            0x5C => {
                self.voices.iter_mut().enumerate().for_each(|(i, c)| {
                    if bit(value, i) {
                        c.adsr_stage = AdsrStage::Release;
                    }
                });
            }
            0x5D => {
                self.sample_dir = (value as usize) << 8;
            }
            0x6C => {
                // Combined mute and reset
                if value & 0xC0 != 0 {
                    self.voices.iter_mut().for_each(|c| c.enabled = false);
                }
                self.echo_enabled = !bit(value, 5);
                self.noise_frequency = PERIOD_TABLE[(value & 0x1F) as usize]
            }
            0x6D => self.echo_addr = (value as usize) << 8,
            0x7D => self.echo_size = 512 * value as usize,
            reg if address & 0x0F < 0x0A => {
                let channel_index = (reg / 0x10) & 0x0F;
                if channel_index < self.voices.len() {
                    self.voices[channel_index].write(address, value);
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
                debug!("Unknown DSP register {address:02X} value={value:02X}");
            }
        }
    }
    pub fn read(&mut self, address: usize) -> u8 {
        match address % 0x80 {
            0x0D => self.echo_feedback as u8,
            0x2C => self.echo_volume[0] as u8,
            0x3C => self.echo_volume[1] as u8,
            0x4D => self
                .voices
                .iter()
                .enumerate()
                .map(|(i, c)| u8::from(c.echo_enabled) << i)
                .sum(),
            0x6D => (self.echo_addr >> 8) as u8,
            0x7C => self
                .voices
                .iter()
                .enumerate()
                .map(|(i, v)| u8::from(v.end_flag) >> (7 - i))
                .sum(),
            0x7D => (self.echo_size / 512) as u8,
            reg if address & 0x0F < 0x0A => {
                let index = (reg / 0x10) & 0x0F;
                if index < self.voices.len() {
                    self.voices[index].read(address)
                } else {
                    0
                }
            }
            _ => todo!(),
        }
    }
    pub fn generate_sample(&mut self, ram: &mut [u8]) {
        let mut prev_pitch: i32 = 0;
        // Clock noise
        self.noise_index = if self.noise_frequency == 0 {
            0
        } else {
            (self.noise_index + 1) % self.noise_frequency
        };
        // Generate noise value
        let noise_val = Random::from_seed(Seed::unsafe_new(self.noise_index as u64)).i32() & 0xFFFF;

        let voices: [[i32; 2]; 8] = core::array::from_fn(|i| {
            self.voices[i].generate_sample(self.sample_dir, &mut prev_pitch, &ram, noise_val)
        });

        let voice_out: [i16; 2] = core::array::from_fn(|side| {
            (voices.iter().map(|arr| arr[side]).sum::<i32>() / self.voices.len() as i32) as i16
        });

        // Do for left/right
        let echo_out: [i16; 2] = core::array::from_fn(|side| {
            if self.echo_enabled {
                // Compute value of FIR taps
                let fir_val: f32 = (0..8)
                    .map(|j| {
                        (self.fir_coeffs[j] as f32 / 128.0)
                            * self.fir_cache[(self.fir_index + j) % self.fir_cache.len()][side]
                                as f32
                    })
                    .sum::<f32>();
                // Compute value of voices with echo enabled
                let echo_voices: i32 = voices
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| self.voices[*i].echo_enabled)
                    .map(|(_, v)| v[side])
                    .sum::<i32>();
                // Multiply FIR tap by feedback value before adding voices
                let output = ((fir_val * self.echo_feedback as f32 / 128.0).floor() as i32
                    + echo_voices) as i16;

                // Write value to memory
                if self.echo_size == 0 {
                    self.fir_cache[self.fir_index][side] = output;
                } else {
                    let index = self.echo_addr + self.echo_index;
                    // Read value from RAM to FIR cache
                    let ram_val = i16::from_le_bytes([ram[index], ram[(index + 1) % ram.len()]]);
                    self.fir_cache[self.fir_index][side] = ram_val;
                    // Write current output to RAM
                    let to_write = output as i16;
                    to_write
                        .to_le_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, v)| {
                            ram[(index + i) % ram.len()] = *v;
                        });
                    // Increment by 2 since we added 2 bytes
                    self.echo_index = (self.echo_index + 2) % self.echo_size;
                }
                output / 128
            } else {
                0
            }
        });
        // Go to next cache value
        self.fir_index = (self.fir_index + 1) % self.fir_cache.len();

        let final_out: [f32; 2] = core::array::from_fn(|side| {
            (echo_out[side] as f32 * self.echo_volume[side] as f32 / 128.0)
                + (voice_out[side] as f32 * self.volume[side] as f32 / 128.0)
        });
        // For right now, just average the left/right sides
        let s = (final_out[0] + final_out[1]) / 2.0 / 0x3FFF as f32;
        if s > 1.0 || s < -1.0 {
            error!("Invalid audio sample generated: {}", s);
            self.sample_queue.push_back(0.0);
        } else {
            self.sample_queue.push_back(s);
        }
    }
}
