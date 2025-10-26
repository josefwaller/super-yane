use std::collections::VecDeque;

use log::debug;

pub const ENVELOPE_MAX_VALUE: u16 = 0x7FF;

pub const RELEASE_PERIOD_RATE: usize = 31;

pub const PERIOD_TABLE: [usize; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32,
    24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];
pub const PERIOD_OFFSET_TABLE: [usize; 32] = [
    0, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040,
    536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0,
];

#[derive(Default, Clone, Copy)]
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

#[derive(Default, Copy, Clone)]
pub enum AdsrStage {
    #[default]
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Default, Clone)]
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
    pub(super) adsr_stage: AdsrStage,
    pub decay_rate: usize,
    pub attack_rate: usize,
    pub sustain_level: u32,
    pub sustain_rate: usize,
    pub gain_rate: usize,
    pub gain_mode: GainMode,
    pub echo_enabled: bool,
    /// Current address of the block being played
    pub(super) block_addr: Option<usize>,
    /// Decoded samples from the BRR block
    pub(super) samples: [i16; 16],
    /// Copy of previous sample data, for gaussian interpolation
    pub(super) prev_sample_data: [i16; 16],
    /// Counter value
    pub(super) counter: u16,
    /// Whether pitch modulation is enabled
    pub(super) pitch_mod_enabled: bool,
    /// The current envelope of the voic
    pub(super) envelope: u16,
    /// Internal period counter value
    period_counter: usize,
}

impl Voice {
    pub fn clock(&mut self, total_clocks: usize) {
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
                        if v <= self.sustain_level as u16 {
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
                    LinearDecrease => self.envelope - 32,
                    ExponentialDecrease => ((self.envelope.saturating_sub(1)) >> 8) + 1,
                    // ExponentialDecrease => todo!("Exponential Decrease"),
                    LinearIncrease => self.envelope + 32,
                    BentIncrease => {
                        if self.envelope < 0x600 {
                            self.envelope + 32
                        } else {
                            self.envelope + 8
                        }
                    }
                }
            }
        }
        self.envelope = self.envelope.clamp(0, ENVELOPE_MAX_VALUE);
    }
    fn get_period_elapsed(&self, rate: usize) -> bool {
        let table_val = PERIOD_TABLE[rate];
        table_val != 0 && (self.period_counter + PERIOD_OFFSET_TABLE[rate]) % table_val == 0
    }
}
