use crate::ppu::utils::{color_to_rgb, rgb_to_color};

#[derive(Clone, Copy, Debug)]
pub enum ColorBlendMode {
    Add,
    Subtract,
    AddHalf,
    SubtractHalf,
}
impl From<u8> for ColorBlendMode {
    fn from(value: u8) -> Self {
        use ColorBlendMode::*;
        match value & 0x03 {
            0 => Add,
            1 => AddHalf,
            2 => Subtract,
            3 => SubtractHalf,
            _ => unreachable!(),
        }
    }
}
impl ColorBlendMode {
    pub fn compute(&self, left: u16, right: u16) -> u16 {
        use ColorBlendMode::*;
        let l_rgb = color_to_rgb(left);
        let r_rgb = color_to_rgb(right);
        let c = core::array::from_fn(|i| {
            let l = l_rgb[i];
            let r = r_rgb[i];
            match self {
                Add => l.saturating_add(r),
                Subtract => l.saturating_sub(r),
                AddHalf => l.saturating_add(r) / 2,
                SubtractHalf => l.saturating_sub(r) / 2,
            }
            .clamp(0, 31)
        });
        rgb_to_color(c)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum ColorMathSource {
    Fixed,
    Subscreen,
}
