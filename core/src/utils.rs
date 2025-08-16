use std::ops::{BitAnd, Shl};

pub fn bit(value: u8, n: usize) -> bool {
    value.bitand(1u8.shl(n)) != 0
}
/// Split a color up into its RGB components
pub fn color_to_rgb(color: u16) -> [u16; 3] {
    [color & 0x1F, (color >> 5) & 0x1F, (color >> 10) & 0x1F]
}
/// Build a color from its RGB components
pub fn rgb_to_color(rgb: [u16; 3]) -> u16 {
    rgb[0] as u16 + rgb[1] as u16 * 0x20 + rgb[2] as u16 * 0x400
}
