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
    /// The byte counter if using a regular DMA, or the
    /// indirect table address if using HDMA
    pub byte_counter: u16,
    /// The byte counter at the moment hte DMA is triggered
    pub(crate) init_byte_counter: u16,
    /// Whether the DMA is being executed right now
    pub(crate) is_executing: bool,
    /// The line counter, if enabled as HDMA
    pub hdma_line_counter: Option<u8>,
    /// Bank of the indirect HDMA data.
    /// If using an indirect HDMA table, this is the bank of the data's address.
    pub hdma_bank: u8,
    /// Address of the HDMA table
    pub hdma_current_table_addr: usize,
    pub hdma_repeat: bool,
    pub hdma_enable: bool,
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
            hdma_line_counter: None,
            hdma_bank: 0,
            hdma_current_table_addr: 0,
            hdma_repeat: false,
            hdma_enable: false,
        }
    }
}
impl Channel {
    pub fn hdma_table_addr(&self) -> usize {
        self.src_bank as usize * 0x10000 + self.hdma_current_table_addr
    }
    pub fn full_src_addr(&self) -> usize {
        self.src_bank as usize * 0x10000 + self.src_addr as usize
    }
    pub fn hdma_indirect_table_addr(&self) -> usize {
        // Byte counter registers are also used for HDMA table address
        self.hdma_bank as usize * 0x10000 + self.byte_counter as usize
    }
    pub fn is_hdma(&self) -> bool {
        self.hdma_enable
    }
}
