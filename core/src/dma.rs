use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub enum AddressAdjustMode {
    #[default]
    Increment,
    Decrement,
    Fixed,
}

impl From<AddressAdjustMode> for u8 {
    fn from(value: AddressAdjustMode) -> Self {
        use AddressAdjustMode::*;
        match value {
            Increment => 0,
            Decrement => 2,
            Fixed => 1,
        }
    }
}

const TRANSFER_PATTERS: &[&[u8]] = &[
    &[0],
    &[0, 1],
    &[0, 0],
    &[0, 0, 1, 1],
    &[0, 1, 2, 3],
    &[0, 1, 0, 1],
    &[0, 0],
    &[0, 0, 1, 1],
];

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(default)]
    pub transfer_pattern_index: usize,
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
    pub indirect_bank: u8,
    /// Current address of the indirect data, if used
    pub indirect_data_addr: u16,
    /// Address of the HDMA table
    pub hdma_table_addr: u16,
    /// Bank of the HDMA table
    pub hdma_table_bank: u8,
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
            transfer_pattern_index: 0,
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
            indirect_bank: 0,
            hdma_table_bank: 0,
            hdma_table_addr: 0,
            current_hdma_table_addr: 0,
            indirect_data_addr: 0,
            hdma_repeat: false,
            hdma_enable: false,
        }
    }
}
impl Channel {
    pub fn transfer_pattern(&self) -> &[u8] {
        TRANSFER_PATTERS[self.transfer_pattern_index]
    }
    pub fn current_hdma_table_addr(&self, offset: u16) -> usize {
        self.hdma_table_bank as usize * 0x1_0000
            + self.current_hdma_table_addr.wrapping_add(offset) as usize
    }
    pub fn inc_table_addr(&mut self) {
        self.current_hdma_table_addr += if self.indirect {
            3
        } else {
            1 + self.transfer_pattern().len() as u16
        };
    }
    pub fn full_src_addr(&self) -> usize {
        return if self.indirect {
            self.indirect_bank
        } else {
            self.src_bank
        } as usize
            * 0x10000
            + if self.indirect {
                self.indirect_data_addr
            } else {
                self.src_addr
            } as usize;
    }
    pub fn is_hdma(&self) -> bool {
        self.hdma_enable
    }
    pub fn get_num_bytes(&self) -> u16 {
        if self.is_hdma() {
            self.transfer_pattern().len() as u16
        } else {
            self.byte_counter
        }
    }
    /// Increment the source by 1 byte every time a byte is DMAed
    pub fn inc_src_addr(&mut self) {
        if self.indirect {
            self.indirect_data_addr = self.indirect_data_addr.wrapping_add(1);
        } else {
            if self.is_hdma() {
                self.src_addr = self.src_addr.wrapping_add(1);
            } else {
                match self.adjust_mode {
                    AddressAdjustMode::Increment => self.src_addr = self.src_addr.wrapping_add(1),
                    AddressAdjustMode::Decrement => self.src_addr = self.src_addr.wrapping_sub(1),
                    AddressAdjustMode::Fixed => {}
                }
            }
        }
    }
}
