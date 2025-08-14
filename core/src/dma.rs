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
    /// How many bytes have been transferred so far
    pub(crate) num_bytes_transferred: u16,
    /// Whether the DMA is being executed right now
    pub(crate) is_executing: bool,
    /// The line counter, if enabled as HDMA
    pub hdma_line_counter: u8,
    /// Bank of the indirect HDMA data.
    /// If using an indirect HDMA table, this is the bank of the data's address.
    pub hdma_bank: u8,
    /// Address of the HDMA table
    pub hdma_table_addr: u16,
    /// Current address of the HDMA table
    /// This should always point to the next table entry to be read.
    /// At the end of VBlank it is initialized to [`Channel::hdma_table_addr`]
    pub current_hdma_table_addr: u16,
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
            num_bytes_transferred: 0,
            is_executing: false,
            hdma_line_counter: 0,
            hdma_bank: 0,
            hdma_table_addr: 0,
            current_hdma_table_addr: 0,
            hdma_repeat: false,
            hdma_enable: false,
        }
    }
}
impl Channel {
    pub fn hdma_table_addr(&self) -> usize {
        self.src_bank as usize * 0x10000 + self.current_hdma_table_addr as usize
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
    pub fn get_num_bytes(&self) -> u16 {
        if self.is_hdma() {
            self.transfer_pattern.len() as u16
        } else {
            self.byte_counter
        }
    }
    pub fn inc_src_addr(&mut self) {
        if self.is_hdma() {
            self.src_addr = self.src_addr.wrapping_add(1);
        } else {
            match self.adjust_mode {
                AddressAdjustMode::Increment => self.src_addr = self.src_addr.wrapping_add(1),
                AddressAdjustMode::Decrement => self.src_addr = self.src_addr.wrapping_sub(1),
                _ => {}
            }
        }
    }
}
