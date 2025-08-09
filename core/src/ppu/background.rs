use std::collections::VecDeque;

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
        }
    }
}
