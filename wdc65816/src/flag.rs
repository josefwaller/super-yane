use std::ops::BitAnd;

/// Utility struct that can act as a binary number (0 or 1) or a boolean.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flag {
    value: u32,
}

impl PartialEq<bool> for Flag {
    fn eq(&self, other: &bool) -> bool {
        (self.value == 1) == *other
    }
    fn ne(&self, other: &bool) -> bool {
        (self.value == 0) == *other
    }
}
impl PartialEq<u32> for Flag {
    fn eq(&self, other: &u32) -> bool {
        #[cfg(debug_assertions)]
        if *other > 1 {
            panic!("Flag value must be 0 or 1, got {}", other);
        }
        self.value == *other
    }
    fn ne(&self, other: &u32) -> bool {
        #[cfg(debug_assertions)]
        if *other > 1 {
            panic!("Flag value must be 0 or 1, got {}", other);
        }
        self.value != *other
    }
}
impl BitAnd for Flag {
    type Output = Flag;
    fn bitand(self, rhs: Self) -> Self::Output {
        Flag::from(self.into() && rhs.into())
    }
}
impl From<bool> for Flag {
    fn from(value: bool) -> Self {
        Flag {
            value: if value { 1 } else { 0 },
        }
    }
}
impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        flag.value == 1
    }
}
impl From<u32> for Flag {
    fn from(value: u32) -> Self {
        #[cfg(debug_assertions)]
        if value > 1 {
            panic!("Flag value must be 0 or 1, got {}", value);
        }
        Flag { value }
    }
}
impl From<Flag> for u32 {
    fn from(flag: Flag) -> Self {
        flag.value
    }
}
impl From<u16> for Flag {
    fn from(value: u16) -> Self {
        u32::from(value).into()
    }
}
impl From<Flag> for u16 {
    fn from(flag: Flag) -> Self {
        u32::from(flag) as u16
    }
}
impl From<u8> for Flag {
    fn from(value: u8) -> Self {
        u32::from(value).into()
    }
}
impl From<Flag> for u8 {
    fn from(flag: Flag) -> Self {
        u32::from(flag) as u8
    }
}
