use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub x: usize,
    pub y: usize,
    pub tile_index: usize,
    // 0 or 1, to select the nametable
    pub name_select: usize,
    pub flip_x: bool,
    pub flip_y: bool,
    pub priority: usize,
    pub palette_index: usize,
    pub size_select: usize,
    pub msb_x: bool,
}
