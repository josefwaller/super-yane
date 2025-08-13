use std::ops::{BitAnd, Shl};

pub fn bit(value: u8, n: usize) -> bool {
    value.bitand(1u8.shl(n)) != 0
}
