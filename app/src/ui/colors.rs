use egui::Color32;

// Better version of hex_color that parses rust int
macro_rules! from_hex {
    ($hex: expr) => {{
        #[cfg(debug_assertions)]
        assert!($hex <= 0xFFFFFF);
        Color32::from_rgb(
            (($hex >> 16) & 0xFF) as u8,
            (($hex >> 8) & 0xFF) as u8,
            ($hex & 0xFF) as u8,
        )
    }};
}

pub const RED_PRIMARY: Color32 = from_hex!(0xff3b48);
pub const RED_SECONDARY: Color32 = from_hex!(0xed878e);
pub const COLOR_ORANGE: Color32 = from_hex!(0xf7cf97);
pub const LIGHT_BLUE_PRIMARY: Color32 = from_hex!(0x2dc1f7);
pub const COLOR_LIGHT_BLUE: Color32 = from_hex!(0x97eef7);
