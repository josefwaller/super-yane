#[derive(Copy, Clone, Default, Debug)]
pub enum AddressAdjustMode {
    #[default]
    Increment,
    Decrement,
    Fixed,
}

#[derive(Clone)]
pub struct Channel {
    pub transfer_pattern: Vec<u32>,
    pub adjust_mode: AddressAdjustMode,
    pub indirect: bool,
    pub direction: bool,
    pub dest_addr: usize,
    /// The lower 16 bits of the DMA address
    pub src_addr: u16,
    /// The bank of the DMA address
    pub src_bank: u8,
    /// The byte counter, or number of bytes to transfer
    pub byte_counter: u16,
    /// The byte counter at the moment hte DMA is triggered
    pub(crate) init_byte_counter: u16,
    /// Whether the DMA is being executed right now
    pub(crate) is_executing: bool,
}

impl Default for Channel {
    fn default() -> Self {
        Channel {
            transfer_pattern: vec![0],
            adjust_mode: AddressAdjustMode::Increment,
            indirect: false,
            direction: false,
            dest_addr: 0,
            src_addr: 0,
            src_bank: 0,
            byte_counter: 0,
            init_byte_counter: 0,
            is_executing: false,
        }
    }
}
