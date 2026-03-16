use std::ops::{BitAnd, Shl};

pub fn bit(value: u8, n: usize) -> bool {
    value.bitand(1u8.shl(n)) != 0
}
// 0BBB BBGG GGGR RRRR
/// Split a color up into its RGB components
pub fn color_to_rgb(color: u16) -> [u16; 3] {
    [color & 0x1F, (color >> 5) & 0x1F, (color >> 10) & 0x1F]
}
pub fn color_to_rgb_bytes(color: u16, brightness: u8) -> [u8; 3] {
    let b = brightness as f32 / 0xF as f32;
    macro_rules! channel {
        ($val: expr) => {
            (($val as f32) * b).floor() as u8
        };
    }
    [
        channel!((color << 3) & 0xF8),
        channel!((color >> 2) & 0xF8),
        channel!((color >> 7) & 0xF8),
    ]
}
/// Build a color from its RGB components
pub fn rgb_to_color(rgb: [u16; 3]) -> u16 {
    rgb[0] as u16 + rgb[1] as u16 * 0x20 + rgb[2] as u16 * 0x400
}
