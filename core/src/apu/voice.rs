use std::collections::VecDeque;

#[derive(Default, Clone)]
pub struct Voice {
    pub enabled: bool,
    /// Volume, first left, then right
    pub volume: [u8; 2],
    /// Sample pitch, i.e. rate at which samples are consumed
    pub sample_pitch: u16,
    /// Sample source
    pub sample_src: usize,
    /// ADSR enable
    pub adsr_enabled: bool,
    pub decay_rate: u32,
    pub attack_rate: u32,
    pub sustain: u32,
    pub sustain_rate: u32,
    pub gain_value: u32,
    pub gain_mode: bool,
    /// Current address of the block being played
    pub(super) block_addr: Option<usize>,
    /// Previous two samples, used for decoding filter
    pub(super) prev_samples: [i16; 2],
    /// Decoded samples from the BRR block
    pub(super) samples: [i16; 16],
    /// Counter value
    pub(super) counter: u16,
}
