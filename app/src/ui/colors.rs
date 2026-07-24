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

pub const DARK_GREY: Color32 = from_hex!(0x090909);
pub const WHITE: Color32 = from_hex!(0xFFFFFF);
pub const GREY: Color32 = from_hex!(0x202020);
pub const LIGHT_GREY: Color32 = from_hex!(0x878787);
pub const PINK_PRIMARY: Color32 = from_hex!(0xe92ef0);
pub const RED_PRIMARY: Color32 = from_hex!(0xff3b48);
pub const RED_SECONDARY: Color32 = from_hex!(0xc92c37);
pub const COLOR_ORANGE: Color32 = from_hex!(0xf7cf97);
pub const LIGHT_BLUE_PRIMARY: Color32 = from_hex!(0x2dc1f7);
pub const LIGHT_BLUE_SECONDARY: Color32 = from_hex!(0x97eef7);
pub const GREEN_PRIMARY: Color32 = from_hex!(0x14fc4f);
pub const GREEN_SECONDARY: Color32 = from_hex!(0x69f08b);
