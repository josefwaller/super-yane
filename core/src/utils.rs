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
/// Convert from a direct color encoded byte (BBGG GRRR) and optionally the palette index (bgr)
/// to the regular 15 bit color format (0BBB BBGG GGGR RRRR)
pub fn from_direct_color(byte: u8, palette_index: u8) -> u16 {
    let byte = byte as u16;
    let pal = palette_index as u16;
    let r = byte & 0x07;
    let g = byte & 0x38;
    let b = byte & 0xC0;
    (r << 2) | (g << 4) | (b << 7) | (pal & 0x04) | ((pal & 0x02) << 5) | ((pal & 0x01) << 11)
}
