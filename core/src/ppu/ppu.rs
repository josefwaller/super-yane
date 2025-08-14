use crate::Background;
use crate::ppu::background::WindowMaskLogic;

use crate::utils::bit;
use log::*;

const PIXELS_PER_SCANLINE: usize = 341;
const SCANLINES: usize = 262;

#[derive(Default, Debug, Clone, Copy)]
pub struct Window {
    pub left: usize,
    pub right: usize,
}

#[derive(Default, Debug, Clone, Copy)]
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

#[derive(PartialEq, PartialOrd, Debug, Copy, Clone)]
pub enum VramIncMode {
    /// Increment after reading the high byte or writing the low byte
    HighReadLowWrite = 0,
    /// Increment after reading the low byte or writing the high byte
    LowReadHighWrite = 1,
}
#[derive(Clone)]
pub struct Ppu {
    /// VBlank flag
    pub vblank: bool,
    pub forced_blanking: bool,
    pub brightness: u32,
    pub bg_mode: u32,
    pub bg3_prio: bool,
    pub backgrounds: [Background; 4],
    pub mosaic_size: u32,
    /// Background H off latch
    pub bg_h_off: u32,
    /// Background V off latch
    pub bg_v_off: u32,
    pub obj_main_enable: bool,
    pub obj_subscreen_enable: bool,
    pub vram_increment_amount: usize,
    pub vram_increment_mode: VramIncMode,
    pub vram_addr: usize,
    pub vram_remap: u32,
    pub vram: [u8; 0x10000],
    vram_latch_low: u8,
    vram_latch_high: u8,
    pub cgram: [u16; 0x100],
    pub cgram_addr: usize,
    cgram_latch: Option<u8>,
    /// Screen buffer
    pub screen_buffer: [u16; 256 * 240],
    /// This is not hte dot, should rename
    pub dot: usize,
    pub oam_sizes: [(usize, usize); 2],
    // Internal OAM address
    pub oam_addr: usize,
    pub oam_name_addr: usize,
    pub oam_name_select: usize,
    pub oam_latch: u8,
    pub oam_sprites: [Sprite; 0x80],
    /// Buffers of sprite pixels for the current scanline.
    /// One for every sprite layer, ordered by priority
    oam_buffers: [[Option<u16>; 0x100]; 4],

    pub windows: [Window; 2],
}

impl Default for Ppu {
    fn default() -> Self {
        Ppu {
            vblank: false,
            forced_blanking: false,
            brightness: 4,
            bg_mode: 0,
            bg3_prio: false,
            backgrounds: core::array::from_fn(|_| Background::default()),
            mosaic_size: 1,
            bg_h_off: 0,
            bg_v_off: 0,
            obj_main_enable: false,
            obj_subscreen_enable: false,
            vram_increment_amount: 1,
            vram_increment_mode: VramIncMode::HighReadLowWrite,
            vram_remap: 0,
            vram_addr: 0,
            vram: [0; 0x10000],
            vram_latch_low: 0,
            vram_latch_high: 0,
            cgram: [0; 0x100],
            cgram_addr: 0,
            cgram_latch: None,
            screen_buffer: [0; 256 * 240],
            dot: 0,
            oam_addr: 0,
            oam_name_addr: 0,
            oam_sizes: [(8, 8); 2],
            oam_name_select: 0x1000,
            oam_latch: 0,
            oam_sprites: [Sprite::default(); 0x80],
            oam_buffers: [[None; 0x100]; 4],
            windows: [Window::default(); 2],
        }
    }
}

impl Ppu {
    pub fn read_byte(&self, addr: usize) -> u8 {
        match addr {
            0x4210 => u8::from(self.vblank) << 7,
            _ => 0,
        }
    }
    pub fn read_byte_mut(&mut self, addr: usize, open_bus: u8) -> u8 {
        match addr {
            0x2139 => {
                let val = self.vram_latch_low;
                if self.vram_increment_mode == VramIncMode::LowReadHighWrite {
                    self.refresh_vram_latch();
                    self.inc_vram_addr();
                }
                val
            }
            0x213A => {
                let val = self.vram_latch_high;
                if self.vram_increment_mode == VramIncMode::HighReadLowWrite {
                    self.refresh_vram_latch();
                    self.inc_vram_addr();
                }
                val
            }
            // Todo: This shouldn't be in PPU
            0x4210 => {
                let v = u8::from(self.vblank) << 7;
                self.vblank = false;
                return v | (open_bus & 0x70);
            }
            _ => 0,
        }
    }
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        macro_rules! window_settings {
            ($off: expr) => {{
                (0..2).for_each(|i| {
                    (0..2).for_each(|j| {
                        self.backgrounds[$off + i].window_invert[j] = bit(value, 4 * i + 2 * j);
                        self.backgrounds[$off + i].window_enabled[j] = bit(value, 4 * i + 2 * j + 1)
                    })
                })
            }};
        }
        match addr {
            0x2100 => {
                self.forced_blanking = bit(value, 3);
                self.brightness = (value & 0x07) as u32;
            }
            0x2101 => {
                self.oam_name_addr = (value as usize & 0x03) << 13;
                self.oam_name_select = ((value as usize & 0x18) + 0x08) << (12 - 3);
                let size_select = (value & 0xE0) >> 5;
                self.oam_sizes = [
                    match size_select {
                        (0..=2) => (8, 8),
                        (3..=4) => (16, 16),
                        5 => (32, 32),
                        (6..=7) => (16, 32),
                        _ => unreachable!("Should be impossible. Size select is {}", size_select),
                    },
                    match size_select {
                        0 => (16, 16),
                        1 | 3 | 7 => (32, 32),
                        2 | 4 | 5 => (64, 64),
                        6 => (32, 64),
                        _ => unreachable!("Should be impossible. Size select is {}", size_select),
                    },
                ];
            }
            0x2102 => {
                self.oam_addr = (self.oam_addr & 0x200) | (2 * value as usize);
            }
            0x2103 => self.oam_addr = (self.oam_addr & 0x0FF) | 0x200 * (value as usize & 0x01),
            0x2104 => {
                if self.oam_addr % 2 == 0 {
                    self.oam_latch = value;
                } else {
                    if self.oam_addr < 0x200 {
                        // Writes only on the second write (oam_addr is odd)
                        self.write_oam_byte(self.oam_addr.wrapping_sub(1) % 0x200, self.oam_latch);
                        self.write_oam_byte(self.oam_addr, value);
                    }
                }
                if self.oam_addr >= 0x200 {
                    // Writes immediately
                    self.write_oam_byte(self.oam_addr, value);
                }
                self.oam_addr = (self.oam_addr + 1) % 0x10000;
            }
            0x2105 => {
                // Copy background sizes
                (0..4).for_each(|i| {
                    self.backgrounds[i].tile_size = if bit(value, i + 4) { 16 } else { 8 };
                });
                self.bg3_prio = (value & 0x08) != 0;
                self.bg_mode = (value & 0x07) as u32;
            }
            0x2106 => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].mosaic = bit(value, i);
                });
                self.mosaic_size = (value & 0xF0) as u32 / 0x10 + 1;
            }
            0x2107..=0x210A => {
                let b = &mut self.backgrounds[addr - 0x2107];
                b.num_horz_tilemaps = if !bit(value, 0) { 1 } else { 2 };
                b.num_vert_tilemaps = if !bit(value, 1) { 1 } else { 2 };
                b.tilemap_addr = ((value & 0xFC) as usize) << 8;
            }
            0x210B => {
                self.backgrounds[0].chr_addr = (value as usize & 0x0F) << 12;
                self.backgrounds[1].chr_addr = (value as usize & 0xF0) << (12 - 4);
            }
            0x210C => {
                self.backgrounds[2].chr_addr = (value as usize & 0x0F) << 12;
                self.backgrounds[3].chr_addr = (value as usize & 0xF0) << (12 - 4);
            }
            0x210D..=0x2114 => {
                let n = (addr - 0x210D) / 2;
                if addr % 2 == 1 {
                    // Horizontal offset
                    let b = &mut self.backgrounds[n];
                    b.h_off =
                        ((value as u32 * 0x100) | (self.bg_v_off & !0x07) | (self.bg_h_off & 0x07))
                            & 0x03FF;
                    self.bg_h_off = value as u32;
                    self.bg_v_off = value as u32;
                } else {
                    // Vertical offset
                    let b = &mut self.backgrounds[n];
                    b.v_off = ((value as u32 * 0x100) | self.bg_v_off) & 0x03FF;
                    self.bg_v_off = value as u32;
                }
            }
            0x2115 => {
                self.vram_increment_amount = match value & 0x03 {
                    0 => 1,
                    1 => 32,
                    2 | 3 => 128,
                    _ => unreachable!("Invalud VRAM increment amount value: {:X}", value),
                };
                self.vram_increment_mode = match bit(value, 7) {
                    false => VramIncMode::HighReadLowWrite,
                    true => VramIncMode::LowReadHighWrite,
                    _ => unreachable!("Should never happen"),
                };
                self.vram_remap = (value >> 2) as u32 & 0x03;
            }
            0x2116 => {
                self.vram_addr = (self.vram_addr & 0x7F00) | (value as usize);
                self.refresh_vram_latch();
            }
            0x2117 => {
                self.vram_addr = (self.vram_addr & 0x00FF) | (value as usize * 0x100) & 0x7FFF;
                self.refresh_vram_latch();
            }
            0x2118 => {
                let remapped_addr = self.remapped_vram_addr();
                // Write the low byte
                self.vram[2 * remapped_addr] = value;
                if self.vram_increment_mode == VramIncMode::HighReadLowWrite {
                    self.inc_vram_addr();
                }
            }
            0x2119 => {
                let remapped_addr = self.remapped_vram_addr();
                // Write the high byte
                self.vram[2 * remapped_addr + 1] = value;
                if self.vram_increment_mode == VramIncMode::LowReadHighWrite {
                    self.inc_vram_addr();
                }
            }
            0x2121 => {
                self.cgram_addr = value as usize;
                self.cgram_latch = None;
            }
            0x2122 => match self.cgram_latch {
                Some(data) => {
                    self.cgram[self.cgram_addr] = (value as u16 * 0x100) + data as u16;
                    self.cgram_latch = None;
                    self.cgram_addr = (self.cgram_addr + 1) % self.cgram.len();
                }
                None => self.cgram_latch = Some(value),
            },
            0x2123 => window_settings!(0),
            0x2124 => window_settings!(2),
            0x2126 => self.windows[0].left = value as usize,
            0x2127 => self.windows[0].right = value as usize,
            0x2128 => self.windows[1].left = value as usize,
            0x2129 => self.windows[1].right = value as usize,
            0x212A => {
                self.backgrounds.iter_mut().enumerate().for_each(|(i, b)| {
                    b.window_mask_logic = WindowMaskLogic::from(value >> (2 * i))
                })
            }
            0x212C => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].main_screen_enable = bit(value, i);
                });
                self.obj_main_enable = bit(value, 4);
            }
            0x212D => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].sub_screen_enable = bit(value, i);
                });
                self.obj_subscreen_enable = bit(value, 4);
            }
            0x212E => self
                .backgrounds
                .iter_mut()
                .enumerate()
                .for_each(|(i, b)| b.windows_enabled_main = bit(value, i)),
            0x212F => self
                .backgrounds
                .iter_mut()
                .enumerate()
                .for_each(|(i, b)| b.windows_enabled_sub = bit(value, i)),
            // Todo
            0x2133 => {} // _ => debug!("Writing {:X} to {:X}, not handled", value, addr),
            0x213B => debug!("Writing to CGRAM read"),
            _ => {}
        }
    }
    fn inc_vram_addr(&mut self) {
        self.vram_addr = (self.vram_addr + self.vram_increment_amount) % 0x8000;
    }
    fn refresh_vram_latch(&mut self) {
        self.vram_latch_low = self.read_vram_byte(self.vram_addr);
        self.vram_latch_high = self.read_vram_byte(self.vram_addr + 1);
    }
    fn read_vram_byte(&self, byte_addr: usize) -> u8 {
        self.vram[byte_addr % self.vram.len()]
    }
    fn remapped_vram_addr(&self) -> usize {
        let addr = match self.vram_remap {
            0 => self.vram_addr,
            1 => {
                (self.vram_addr & 0xFF00) + (self.vram_addr >> 5)
                    & 0x07 + (self.vram_addr << 3)
                    & 0xF8
            }
            2 => {
                (self.vram_addr & 0xFE00) + (self.vram_addr >> 6)
                    & 0x07 + (self.vram_addr << 4)
                    & 0x01C0
            }
            3 => {
                (self.vram_addr & 0xFC00) + (self.vram_addr >> 7)
                    & 0x07 + (self.vram_addr << 5)
                    & 0x0380
            }
            _ => unreachable!("Invalid VRAM REMAP value: {:X}", self.vram_remap),
        };
        addr & 0x7FFF
    }
    /// Write a single byte to OAM
    fn write_oam_byte(&mut self, addr: usize, value: u8) {
        let addr = addr % (0x220);
        if addr < 0x200 {
            let sprite_index = (addr / 4) % self.oam_sprites.len();
            let sprite = &mut self.oam_sprites[sprite_index];
            match addr % 4 {
                0 => sprite.x = value as usize,
                1 => sprite.y = value as usize,
                2 => {
                    sprite.tile_index = value as usize;
                }
                3 => {
                    sprite.flip_y = (value & 0x80) != 0;
                    sprite.flip_x = (value & 0x40) != 0;
                    sprite.priority = ((value & 0x30) >> 4) as usize;
                    sprite.palette_index = (value >> 1) as usize & 0x07;
                    sprite.name_select = value as usize & 0x01;
                }
                _ => unreachable!(),
            }
        } else {
            let index = (addr - 0x200) * 4;
            (0..4).for_each(|i| {
                let d = value >> (2 * i);
                let s = &mut self.oam_sprites[index + i];
                s.msb_x = d & 0x01 != 0;
                s.size_select = if d & 0x02 == 0 { 0 } else { 1 };
            })
        }
    }
    fn get_2bpp_slice_at(&self, addr: usize) -> [u8; 8] {
        let low = self.vram[addr % self.vram.len()];
        let high = self.vram[(addr + 1) % self.vram.len()];
        core::array::from_fn(|i| ((low >> (7 - i)) & 0x01) + 2 * ((high >> (7 - i)) & 0x01))
    }
    fn extend_background_byte_buffer(&mut self, index: usize, (x, y): (usize, usize), bpp: usize) {
        // Get an immutable reference to the background
        let b = &self.backgrounds[index];
        // Get the tilemaps to render, relative to the current tilemap address
        // So thi is basically an offset to add to the tilemap address
        // [top left, top right, bot left, bot right]
        let mirrored_tile_addrs = if b.num_horz_tilemaps == 2 {
            if b.num_vert_tilemaps == 2 {
                [0, 1, 2, 3]
            } else {
                [0, 1, 0, 1]
            }
        } else {
            if b.num_vert_tilemaps == 2 {
                [0, 0, 1, 1]
            } else {
                [0, 0, 0, 0]
            }
        };
        // Get the tilemap address and X/Y coord of the pixel in the tilemap
        let (tilemap_addr, x, y) = {
            let x = (x + b.h_off as usize) % 512;
            let y = (y + b.v_off as usize) % 512;
            const WORDS_PER_TILEMAP: usize = 32 * 32;
            if x >= 256 {
                if y >= 256 {
                    (
                        b.tilemap_addr + mirrored_tile_addrs[3] * WORDS_PER_TILEMAP,
                        x % 256,
                        y % 256,
                    )
                } else {
                    (
                        b.tilemap_addr + mirrored_tile_addrs[1] * WORDS_PER_TILEMAP,
                        x % 256,
                        y,
                    )
                }
            } else if y >= 256 {
                (
                    b.tilemap_addr + mirrored_tile_addrs[2] * WORDS_PER_TILEMAP,
                    x,
                    y % 256,
                )
            } else {
                (b.tilemap_addr, x, y)
            }
        };
        // Calculate what tile we are drawing
        let tile_x = x / 8;
        let tile_y = y / 8;
        // 2 bytes/tile, 32 tiles/row
        // Note that there's always 2 bytes per tile of the TILEMAP, regardless of how many bpp the tile will use
        let addr = 2 * (32 * tile_y + tile_x);
        // Load the tile
        let tile_addr = 2 * tilemap_addr + addr;
        let tile_low = self.vram[tile_addr % self.vram.len()];
        let tile_high = self.vram[(tile_addr + 1) % self.vram.len()];
        let tile_index = tile_low as usize + 0x100 * (tile_high as usize & 0x03);
        let palette_index = (tile_high as usize & 0x1C) >> 2;
        let priority = tile_high & 0x20 != 0;
        let flip_x = tile_high & 0x40 != 0;
        let flip_y = tile_high & 0x80 != 0;

        let fine_y = if flip_y { 7 - y % 8 } else { y % 8 };
        let slice_addr = (2 * b.chr_addr + 2 * fine_y as usize + (bpp * 8 * tile_index as usize))
            % self.vram.len();
        // Get all the slices
        let slices = (0..(bpp as usize / 2))
            .map(|i| self.get_2bpp_slice_at(slice_addr + 16 * i))
            .collect::<Vec<[u8; 8]>>();
        // Todo: Make this actually change
        let direct_color = false;
        let temp: [u16; 256] = core::array::from_fn(|i| {
            let i = i as u16;
            let r = i & 0x07;
            let g = i & 0x38;
            let b = i & 0xC0;
            (r << 2) | (g << 6) | (b << 7)
        });

        // palette_index is at most 7, so the highest index is (16 * 7 + 16 - 1) = 127
        let palette = match bpp {
            2 => {
                let i = if self.bg_mode == 0 { index } else { 0 };
                &self.cgram[(4 * 8 * i + 4 * palette_index)..(4 * 8 * i + 4 * palette_index + 4)]
            }
            4 => &self.cgram[(16 * palette_index)..(16 * palette_index + 16)],
            8 => {
                if direct_color {
                    &temp
                } else {
                    &self.cgram
                }
            }
            _ => panic!("Unsupported bpp: {}", bpp),
        };
        // Get a mutable reference to the background now
        let b = &mut self.backgrounds[index];
        // We "skip" the first (x % 8) pixels
        // Since each byte contains data for 8 consecutive pixels
        // if the screen is scrolled over horizontally by less than 8 pixels
        // (or any amount that isn't a multiple of 8), we need to load the
        // byte and then only use some of the data it in
        // So we skip the first (x % 8) pixels by starting with that offset
        ((x % 8)..8).for_each(|i| {
            b.pixel_buffer.push_back({
                let v = slices
                    .iter()
                    .enumerate()
                    .map(|(j, s)|
                    // Shifted left by 2 since each slice will have 2 bits per pixel
                    (s[if flip_x {7 - i } else { i }] as usize) << (2 * j))
                    .sum::<usize>();
                if v == 0 {
                    None
                } else {
                    Some((palette[v], priority))
                }
            })
        });
    }
    fn reset_oam_buffer(&mut self, y: usize) {
        // Todo: Don't copy this every scanline
        let mut buffers = [[None; 0x100]; 4];
        self.oam_sprites.iter().for_each(|s| {
            let size = self.oam_sizes[s.size_select];
            if s.y <= y && s.y + size.1 > y {
                let (fine_y, tile_y) = if s.flip_y {
                    (7 - (y - s.y) % 8, (size.1 - 1 - (y - s.y)) / 8)
                } else {
                    ((y - s.y) % 8, (y - s.y) / 8)
                };
                let tile_index = s.tile_index + 16 * tile_y;
                let slice_addr = 2 * (self.oam_name_addr + s.name_select * self.oam_name_select)
                    + 32 * tile_index;
                let width = size.0;
                // Todo: Optimize this so that we don't fetch all the tiles all the time
                let tile_lows: [[u8; 8]; 8] = core::array::from_fn(|i| {
                    if i < width / 8 {
                        self.get_2bpp_slice_at(slice_addr + 32 * i + 2 * fine_y)
                    } else {
                        [0; 8]
                    }
                });
                let tile_highs: [[u8; 8]; 8] = core::array::from_fn(|i| {
                    if i < width / 8 {
                        self.get_2bpp_slice_at(slice_addr + 32 * i + 2 * fine_y + 16)
                    } else {
                        [0; 8]
                    }
                });
                let palette_index = 0x80 + 0x10 * s.palette_index;
                let palette = &self.cgram[palette_index..(palette_index + 0x10)];
                (0..width).for_each(|i| {
                    // Check if the pixel at (sprite x + i) is on the screen
                    let sx = s.x as i32 + i as i32 + if s.msb_x { -0x100 } else { 0 };
                    if sx < 0x100 && sx > 0x00 {
                        // Get slice to draw
                        let (tile_low, tile_high, x) = if s.flip_x {
                            (
                                tile_lows[(size.0 - 1 - i) / 8],
                                tile_highs[(size.0 - 1 - i) / 8],
                                7 - i % 8,
                            )
                        } else {
                            (tile_lows[i / 8], tile_highs[i / 8], i % 8)
                        };
                        let p = tile_low[x] as usize + 4 * tile_high[x] as usize;
                        // Add this sprite's data to the scanline
                        let buf = &mut buffers[s.priority];
                        buf[sx as usize] =
                            buf[sx as usize].or(if p == 0 { None } else { Some(palette[p]) });
                    }
                })
            }
        });
        self.oam_buffers = buffers;
    }
    pub fn advance_master_clock(&mut self, clock: u32) {
        (0..clock).for_each(|_| {
            self.dot = (self.dot + 1) % (4 * (PIXELS_PER_SCANLINE * SCANLINES));
            if self.dot % 4 == 0 {
                // Note the visual picture starts at dot 88
                let x = (self.dot / 4).wrapping_sub(22) % PIXELS_PER_SCANLINE;
                let y = (self.dot / 4) / PIXELS_PER_SCANLINE;
                // Todo: check if this timing is correct
                if y == 0 && x == 0 {
                    self.vblank = false;
                }
                if y == 241 && x == 0 {
                    self.vblank = true;
                }
                if x == 0 {
                    // Clear out data from previous line
                    self.backgrounds
                        .iter_mut()
                        .for_each(|b| b.pixel_buffer.clear());
                    // Compute all data for sprites on this line
                    // Todo: figure out the exact timing of this
                    if y > 0 {
                        self.reset_oam_buffer(y - 1);
                    }
                }
                if x < 256 && y < 240 {
                    let window_vals: [bool; 2] = core::array::from_fn(|i| {
                        self.windows[i].left <= x && x <= self.windows[i].right
                    });
                    // Structured (background_number, bpp)
                    let backgrounds = match self.bg_mode {
                        0 => [(0, 2), (1, 2), (2, 2), (3, 2)].to_vec(),
                        1 => [(0, 4), (1, 4), (2, 2)].to_vec(),
                        3 => [(0, 8), (1, 4)].to_vec(),
                        5 => [(0, 4), (1, 2)].to_vec(),
                        _ => todo!("Background mode {} not implemented", self.bg_mode),
                    };
                    for (i, bpp) in backgrounds.iter() {
                        if self.backgrounds[*i].pixel_buffer.is_empty() {
                            self.extend_background_byte_buffer(*i, (x, y), *bpp);
                        }
                    }
                    let bg_pixels: Vec<Option<(u16, bool)>> = backgrounds
                        .iter()
                        .map(|(i, _bpp)| {
                            // Should be impossible to there to be no pixels right now
                            let b = &mut self.backgrounds[*i];
                            if b.main_screen_enable {
                                b.pixel_buffer.pop_front().unwrap()
                            } else {
                                None
                            }
                        })
                        .collect();
                    // Get the pixel from a background layer with a given priority, or None
                    macro_rules! bg {
                        ($index: expr, $priority: expr) => {{
                            let b = &self.backgrounds[$index];
                            let wv: [bool; 2] =
                                core::array::from_fn(|i| window_vals[i] ^ b.window_invert[i]);
                            let v = if b.window_enabled[0] {
                                if b.window_enabled[1] {
                                    b.window_mask_logic.compute(wv[0], wv[1])
                                } else {
                                    wv[0]
                                }
                            } else if b.window_enabled[1] {
                                wv[1]
                            } else {
                                false
                            };
                            if v {
                                None
                            } else {
                                bg_pixels[$index]
                                    .filter(|(_, p)| *p == $priority)
                                    .map(|(v, _)| v)
                            }
                        }};
                    }
                    // Get the pixel from a sprite layer with a given priority, or None
                    macro_rules! spr {
                        ($index: expr) => {
                            self.oam_buffers[$index][x]
                        };
                    }
                    // The pixels at the given dot, in order from front to back
                    // Can get the first non-None pixel to draw and discard the rest (since they will be behind)
                    let in_order_pixels = match self.bg_mode {
                        0 => [
                            spr!(3),
                            bg!(0, true),
                            bg!(1, true),
                            spr!(2),
                            bg!(0, false),
                            bg!(1, false),
                            spr!(1),
                            bg!(2, true),
                            bg!(3, true),
                            spr!(0),
                            bg!(2, false),
                            bg!(3, false),
                        ]
                        .to_vec(),
                        1 => [
                            if self.bg3_prio { bg!(2, true) } else { None },
                            spr!(3),
                            bg!(0, true),
                            bg!(1, true),
                            spr!(2),
                            bg!(0, false),
                            bg!(1, false),
                            spr!(1),
                            if self.bg3_prio { None } else { bg!(2, true) },
                            spr!(0),
                            bg!(2, false),
                        ]
                        .to_vec(),
                        3 => [
                            spr!(3),
                            bg!(0, true),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            bg!(0, false),
                            spr!(0),
                            bg!(1, false),
                        ]
                        .to_vec(),
                        5 => [
                            spr!(3),
                            bg!(0, true),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            bg!(0, false),
                            spr!(0),
                            bg!(1, false),
                        ]
                        .to_vec(),
                        _ => todo!("Background mode {} not implemented", self.bg_mode),
                    };
                    let pixel = in_order_pixels
                        .iter()
                        // todo can probably combine these two lines
                        .find(|bg_pixel| bg_pixel.is_some())
                        .map_or(self.cgram[0], |p| p.unwrap() & 0x7FFF);
                    self.screen_buffer[256 * y + x] = pixel;
                }
            }
        })
    }
    pub fn is_in_vblank(&self) -> bool {
        (self.dot / 4) / PIXELS_PER_SCANLINE > 240
    }
    pub fn is_in_hblank(&self) -> bool {
        (self.dot / 4) % PIXELS_PER_SCANLINE >= 274
    }
    pub fn scanline(&self) -> usize {
        (self.dot / 4) / PIXELS_PER_SCANLINE
    }
}
