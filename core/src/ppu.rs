use std::collections::VecDeque;

use log::*;

const PIXELS_PER_SCANLINE: usize = 341;
const SCANLINES: usize = 262;

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
    pixel_buffer: VecDeque<Option<(u16, bool)>>,
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
    pub vram_increment_amount: u32,
    pub vram_increment_mode: VramIncMode,
    pub vram_addr: usize,
    pub vram: [u8; 0x10000],
    pub cgram: [u16; 0x100],
    pub cgram_addr: usize,
    pub cgram_latch: Option<u8>,
    /// Screen buffer
    pub screen_buffer: [u16; 256 * 240],
    pub dot: usize,
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
            vram_addr: 0,
            vram: [0; 0x10000],
            cgram: [0; 0x100],
            cgram_addr: 0,
            cgram_latch: None,
            screen_buffer: [0; 256 * 240],
            dot: 0,
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
        macro_rules! bit {
            ($bit_num: expr) => {
                (((value as u32) >> ($bit_num)) & 0x01)
            };
        }
        match addr {
            0x2100 => {
                self.forced_blanking = bit!(3) == 1;
                self.brightness = (value & 0x07) as u32;
            }
            0x2105 => {
                // Copy background sizes
                (0..4).for_each(|i| {
                    self.backgrounds[i].tile_size = if bit!(i + 4) == 1 { 16 } else { 8 };
                });
                self.bg3_prio = (value & 0x08) != 0;
                self.bg_mode = (value & 0x07) as u32;
            }
            0x2106 => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].mosaic = bit!(i) == 1;
                });
                self.mosaic_size = (value & 0xF0) as u32 / 0x10 + 1;
            }
            0x2107..=0x210A => {
                let b = &mut self.backgrounds[addr - 0x2107];
                b.num_horz_tilemaps = if bit!(0) == 0 { 1 } else { 2 };
                b.num_vert_tilemaps = if bit!(1) == 0 { 1 } else { 2 };
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
                if addr % 2 == 1 {
                    // Horizontal offset
                    let b = &mut self.backgrounds[(addr - 0x210D) / 2];
                    b.h_off = ((value as u32 * 0x10000)
                        | (self.bg_v_off & !0x07)
                        | (self.bg_h_off & 0x07))
                        & 0x03FF;
                    self.bg_h_off = value as u32;
                    self.bg_v_off = value as u32;
                } else {
                    // Vertical offset
                    let b = &mut self.backgrounds[(addr - 0x210E) / 2];
                    b.v_off = ((value as u32 * 0x10000) | self.bg_v_off) & 0x03FF;
                    self.bg_v_off = value as u32;
                }
            }
            0x2115 => {
                self.vram_increment_amount = match value & 0x03 {
                    0 => 1,
                    1 => 32,
                    _ => 128,
                };
                self.vram_increment_mode = match bit!(7) {
                    0 => VramIncMode::HighReadLowWrite,
                    1 => VramIncMode::LowReadHighWrite,
                    _ => panic!("Should never happen"),
                }
            }
            0x2116 => {
                self.vram_addr = (self.vram_addr & 0x7F00) | (value as usize);
            }
            0x2117 => {
                self.vram_addr = (self.vram_addr & 0x00FF) | (value as usize * 0x100) & 0x7FFF;
            }
            0x2118 => {
                // Write the low byte
                self.vram[2 * self.vram_addr] = value;
                // debug!("Write {:X} to {:X} H", value, self.vram_addr);
                if self.vram_increment_mode == VramIncMode::HighReadLowWrite {
                    self.vram_addr = (self.vram_addr + 1) % 0x8000;
                }
            }
            0x2119 => {
                // Write the high byte
                self.vram[2 * self.vram_addr + 1] = value;
                // debug!("{:X} {:X} L", self.vram_addr, value);
                if self.vram_increment_mode == VramIncMode::LowReadHighWrite {
                    self.vram_addr = (self.vram_addr + 1) % 0x8000;
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
            0x212C => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].main_screen_enable = bit!(i) == 1;
                });
                self.obj_main_enable = bit!(4) == 1;
            }
            0x212D => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].sub_screen_enable = bit!(i) == 1;
                });
                self.obj_subscreen_enable = bit!(4) == 1;
            }
            // Todo
            0x2133 => {} // _ => debug!("Writing {:X} to {:X}, not handled", value, addr),
            0x213B => debug!("Writing to CGRAM read"),
            _ => {}
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
            let x = x + b.h_off as usize;
            let y = y + b.v_off as usize;
            const WORDS_PER_TILEMAP: usize = 32 * 32;
            if x >= 256 {
                if y >= 256 {
                    (b.tilemap_addr + mirrored_tile_addrs[3] * WORDS_PER_TILEMAP, x % 256, y % 256)
                } else {
                    (b.tilemap_addr + mirrored_tile_addrs[1] * WORDS_PER_TILEMAP, x % 256, y)
                }
            } else if y >= 256 {
                (b.tilemap_addr + mirrored_tile_addrs[2] * WORDS_PER_TILEMAP, x, y % 256)
            } else {
                (b.tilemap_addr, x, y)
            }
        };
        // These are here since they will have to change once the background scrolling is implemented
        let tile_x = x / 8;
        let tile_y = y / 8;
        let fine_y = y % 8;
        // 2 bytes/tile, 32 tiles/row
        // Note that there's always 2 bytes per tile of the TILEMAP, regardless of how many bpp the tile will use
        let addr = 2 * (32 * tile_y + tile_x);
        // Load the tile
        let tile_addr = 2 * tilemap_addr + addr;
        let tile_low = self.vram[tile_addr % self.vram.len()];
        let tile_high = self.vram[(tile_addr + 1) % self.vram.len()];
        let tile_index = tile_low as usize + 0x100 * (tile_high as usize & 0x03);
        let palette_index = (tile_high as usize & 0x1C) >> 2;
        let priority = tile_high & 20 != 0;
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
            2 => &self.cgram[(4 * 8 * index + 4 * palette_index)..(4 * 8 * index + 4 * palette_index + 4)],
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
                    (s[i] as usize) << (2 * j))
                    .sum::<usize>();
                if v == 0 {
                    None
                } else {
                    Some((palette[v], priority))
                }
            })
        });
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
                }
                if x < 256 && y < 240 {
                    // Structured (background_number, bpp)
                    let backgrounds = match self.bg_mode {
                        0 => [(0, 2), (1, 2), (2, 2), (3, 2)].to_vec(),
                        3 => [(0, 8), (1, 4)].to_vec(),
                        5 => [(0, 4), (1, 2)].to_vec(),
                        _ => todo!("Background mode {} not implemented", self.bg_mode),
                    };
                    for (i, bpp) in backgrounds.iter() {
                        if self.backgrounds[*i].pixel_buffer.is_empty() {
                            self.extend_background_byte_buffer(*i, (x, y), *bpp);
                        }
                    }
                    let pixels: Vec<Option<(u16, bool)>> = backgrounds
                        .iter()
                        .map(|(i, p)| {
                            // Should be impossible to there to be no pixels right now
                            let b = &mut self.backgrounds[*i];
                            if b.main_screen_enable || b.sub_screen_enable {
                                b.pixel_buffer.pop_front().unwrap()
                            } else {
                                None
                            }
                        })
                        .collect();
                    let order = match self.bg_mode {
                        0 => [
                            (0, true),
                            (1, true),
                            (0, false),
                            (1, false),
                            (2, true),
                            (3, true),
                            (2, false),
                            (3, false),
                        ]
                        .to_vec(),
                        3 => [(0, true), (1, true), (0, false), (1, false)].to_vec(),
                        5 => [(0, true), (1, true), (0, false), (1, false)].to_vec(),
                        _ => todo!("Background mode {} not implemented", self.bg_mode),
                    };
                    let pixel = order
                        .iter()
                        .find(|(i, prio)| pixels[*i].is_some_and(|(v, p)| p == *prio))
                        .map_or(self.cgram[0], |(i, _)| pixels[*i].unwrap().0 & 0x7FFF);
                    self.screen_buffer[256 * y + x] = pixel;
                }
            }
        })
    }
    pub fn is_in_vblank(&self) -> bool {
        self.dot > 88 + 240 * PIXELS_PER_SCANLINE
    }
}
