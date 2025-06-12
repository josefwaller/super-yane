use std::ops::Add;

const OVERFLOW: u32 = 0x1000000;

#[derive(PartialEq, PartialOrd, Eq, Clone, Copy)]
#[allow(nonstandard_style)]
pub struct u24 {
    value: u32,
}

impl u24 {
    pub fn from(bank: u8, lower: u16) -> u24 {
        u24 {
            value: (bank as u32) * 0x10000 + lower as u32,
        }
    }
    pub fn with_bank(&self, bank: u8) -> u24 {
        u24::from(bank, (self.value & 0xFFFF) as u16)
    }
    pub fn wrapping_add<T: Into<u32>>(&self, rhs: T) -> u24 {
        u24 {
            value: self.value.wrapping_add(rhs.into()) % OVERFLOW,
        }
    }
}
impl Add<u32> for u24 {
    type Output = u24;
    fn add(self, rhs: u32) -> Self::Output {
        u24 {
            value: rhs.wrapping_add(self.value) % OVERFLOW,
        }
    }
}

impl From<u32> for u24 {
    fn from(value: u32) -> Self {
        u24 {
            value: value & 0xFFFFFF,
        }
    }
}
impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        value.value
    }
}
impl From<u24> for usize {
    fn from(value: u24) -> Self {
        value.value as usize
    }
}
