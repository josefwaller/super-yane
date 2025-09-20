use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum WindowMaskLogic {
    Or,
    And,
    Xor,
    Xnor,
}

impl From<u8> for WindowMaskLogic {
    fn from(value: u8) -> Self {
        use WindowMaskLogic::*;
        match value & 0x3 {
            0 => Or,
            1 => And,
            2 => Xor,
            3 => Xnor,
            _ => unreachable!(),
        }
    }
}
impl WindowMaskLogic {
    pub fn compute(&self, a: bool, b: bool) -> bool {
        use WindowMaskLogic::*;
        match self {
            Or => a | b,
            And => a & b,
            Xor => a ^ b,
            Xnor => !(a ^ b),
        }
    }
}

#[derive(Clone)]
pub struct Background {
    // 0 = 8x8, 1 = 16x16
    pub tile_size: u32,
    pub mosaic: bool,
    pub num_horz_tilemaps: u32,
    pub num_vert_tilemaps: u32,
    pub tilemap_addr: usize,
    pub chr_addr: usize,
    pub h_off: u32,
    pub v_off: u32,
    pub main_screen_enable: bool,
    pub sub_screen_enable: bool,
    /// Buffer for pixel data, used by PPU to render the background
    pub(super) pixel_buffer: VecDeque<Option<(u16, bool)>>,
    pub window_mask_logic: WindowMaskLogic,
    pub windows_enabled_main: bool,
    pub windows_enabled_sub: bool,
    pub window_enabled: [bool; 4],
    pub window_invert: [bool; 4],
    pub color_math_enable: bool,
    /// Colors of the top-left pixel for each mosaic block.
    /// Not all of these values will be used
    pub mosaic_values: [Option<(u16, bool)>; 256],
}
impl Default for Background {
    fn default() -> Self {
        Background {
            tile_size: 8,
            mosaic: false,
            num_horz_tilemaps: 1,
            num_vert_tilemaps: 1,
            tilemap_addr: 0,
            chr_addr: 0,
            h_off: 0,
            v_off: 0,
            main_screen_enable: false,
            sub_screen_enable: false,
            pixel_buffer: VecDeque::new(),
            window_mask_logic: WindowMaskLogic::And,
            windows_enabled_main: false,
            windows_enabled_sub: false,
            window_enabled: [false; 4],
            window_invert: [false; 4],
            color_math_enable: false,
            mosaic_values: [None; 256],
        }
    }
}
