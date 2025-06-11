use crate::u24::u24;
/// The 24-bit address bus used by the processor to read data
#[derive(Copy, Clone)]
pub struct Bus {
    pub bank: u8,
    pub address: u16,
}
impl Bus {
    pub fn new(bank: u8, address: u16) -> Bus {
        Bus { bank, address }
    }
    /// Creates a new bus with the offset given
    /// Wraps around the lower 16 bit address
    pub fn offset(&self, value: u16) -> Bus {
        Bus::new(self.bank, self.address.wrapping_add(value))
    }
    pub fn with_bank(&self, bank: u8) -> Bus {
        Bus {
            bank,
            address: self.address,
        }
    }
}
impl From<u32> for Bus {
    fn from(value: u32) -> Self {
        Bus::new(((value & 0xFF0000) >> 16) as u8, (value & 0xFFFF) as u16)
    }
}
impl From<Bus> for u32 {
    fn from(value: Bus) -> Self {
        (value.bank as u32) * 0x10000 + (value.address as u32)
    }
}
impl From<Bus> for usize {
    fn from(value: Bus) -> Self {
        u32::from(value) as usize
    }
}
impl From<u24> for Bus {
    fn from(value: u24) -> Self {
        Bus::from(u32::from(value))
    }
}
impl From<Bus> for u24 {
    fn from(value: Bus) -> Self {
        u24::from(u32::from(value))
    }
}
