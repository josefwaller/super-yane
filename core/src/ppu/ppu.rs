use crate::Background;
use crate::{
    ppu::{
        Sprite,
        background::{BackgroundPixel, WindowMaskLogic},
        color_math::{ColorBlendMode, ColorMathSource},
        window::{Window, WindowRegion},
    },
    utils::rgb_to_color,
};

use crate::utils::{bit, color_to_rgb_bytes};
use log::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const PIXELS_PER_SCANLINE: usize = 341;
pub const SCANLINES: usize = 262;

pub fn convert_8p8(value: u16) -> f32 {
    // ((value as i16 >> 8) as f32) + (value & 0xFF) as i8 as f32 / 0x100 as f32
    (value as i16 as f32) / 0x100 as f32
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Matrix {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub center_x: i16,
    pub center_y: i16,
}

#[derive(PartialEq, PartialOrd, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum VramIncMode {
    /// Increment after reading the high byte or writing the low byte
    HighReadLowWrite = 0,
    /// Increment after reading the low byte or writing the high byte
    LowReadHighWrite = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Mode7Fill {
    Transparent = 0,
    Character = 1,
}

fn default_2bpp_cache() -> Box<[[u8; 8]; 0x10000 / 2]> {
    Box::new([[0; 8]; 0x10000 / 2])
}

fn default_oam_buffer() -> [[Option<u16>; 0x100]; 4] {
    [[None; 0x100]; 4]
}

fn default_screen_buffer() -> [u16; 256 * 240] {
    [0; 256 * 240]
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ppu {
    /// VBlank flag
    pub vblank: bool,
    pub forced_blanking: bool,
    pub brightness: u32,
    pub bg_mode: u32,
    pub bg3_prio: bool,
    pub backgrounds: [Background; 4],
    pub mosaic_size: usize,
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
    #[serde(with = "BigArray")]
    pub vram: [u8; 0x10000],
    // Cache of VRAM decoded in 2BPP format
    // Hopefully speeds things up a bit
    #[serde(skip, default = "default_2bpp_cache")]
    vram_cache_2bpp: Box<[[u8; 8]; 0x10000 / 2]>,
    vram_latch_low: u8,
    vram_latch_high: u8,
    #[serde(with = "BigArray")]
    pub cgram: [u16; 0x100],
    pub cgram_addr: usize,
    cgram_latch: Option<u8>,
    /// Screen buffer
    #[serde(skip, default = "default_screen_buffer")]
    pub screen_buffer: [u16; 256 * 240],
    /// This is not hte dot, should rename
    pub dot: usize,
    pub oam_sizes: [(usize, usize); 2],
    // Internal OAM address
    pub oam_addr: usize,
    pub oam_name_addr: usize,
    pub oam_name_select: usize,
    pub oam_latch: u8,
    #[serde(with = "BigArray")]
    pub oam_sprites: [Sprite; 0x80],
    /// Buffers of sprite pixels for the current scanline.
    /// One for every sprite layer, ordered by priority
    #[serde(skip, default = "default_oam_buffer")]
    oam_buffers: [[Option<u16>; 0x100]; 4],
    pub color_blend_mode: ColorBlendMode,
    pub color_math_enable_backdrop: bool,
    pub color_math_enable_obj: bool,
    // RGB format
    pub fixed_color: [u16; 3],
    pub color_math_src: ColorMathSource,
    pub color_window_main_region: WindowRegion,
    pub color_window_sub_region: WindowRegion,
    pub windows: [Window; 2],
    pub color_window_logic: WindowMaskLogic,
    #[serde(default)]
    pub sprite_window_logic: WindowMaskLogic,
    pub direct_color: bool,
    pub overscan: bool,
    // The matrix values (a, b, c, and d), some of which (a, b) are also used for the
    // multiplication result
    pub matrix: Matrix,
    /// Mode 7 horizontal offset
    pub m7_h_off: i16,
    /// Mode 7 vertical offset
    pub m7_v_off: i16,
    /// Mode 7 latch
    pub m7_latch: u8,
    /// Mode 7 tilemap repeat
    pub m7_repeat: bool,
    /// Mode 7 tilemap fill (if not repeating)
    pub m7_fill: Mode7Fill,
    /// Mode 7 flip horizontal
    pub m7_flip_h: bool,
    /// Mode 7 flip vertical
    pub m7_flip_v: bool,
    /// Multiplication latch
    pub multi_latch: u8,
    /// Multiplication result
    pub multi_res: i32,
    /// Vertical count of the current mosaic block.
    /// Basically a countdown (or count up) until when to recompute the mosaic latches
    mosaic_v_latch: usize,
    /// Interlace field, should be toggled every frame
    pub interlace_field: bool,
    /// Last latched horizontal dot position
    pub h_latch: usize,
    /// Last lateched vertical dot position
    pub v_latch: usize,
    /// Latch to determine whether to return the high or low value from dot positions
    pub ophct_latch: bool,
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
            vram_cache_2bpp: Box::new([[0; 8]; 0x10000 / 2]),
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
            color_blend_mode: ColorBlendMode::Add,
            color_math_enable_backdrop: false,
            color_math_enable_obj: false,
            fixed_color: [0; 3],
            color_math_src: ColorMathSource::Fixed,
            color_window_main_region: WindowRegion::Nowhere,
            color_window_sub_region: WindowRegion::Nowhere,
            color_window_logic: WindowMaskLogic::And,
            sprite_window_logic: WindowMaskLogic::And,
            direct_color: false,
            overscan: false,
            matrix: Matrix::default(),
            m7_h_off: 0,
            m7_v_off: 0,
            m7_latch: 0,
            m7_fill: Mode7Fill::Transparent,
            m7_repeat: false,
            m7_flip_v: false,
            m7_flip_h: false,
            multi_latch: 0,
            multi_res: 0,
            mosaic_v_latch: 0,
            interlace_field: false,
            h_latch: 0,
            v_latch: 0,
            ophct_latch: false,
        }
    }
}

impl Ppu {
    pub fn read_byte(&mut self, addr: usize, open_bus: u8) -> u8 {
        match addr {
            0x2134..=0x2136 => self.multi_res.to_le_bytes()[addr - 0x2134],
            0x2137 => {
                let (x, y) = self.dot_xy();
                // Update latches
                self.h_latch = x;
                self.v_latch = y;
                open_bus
            }
            0x2138 => {
                warn!("Read from OAMDATAREAD");
                0
            }
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
            0x213B => {
                self.cgram_latch = match self.cgram_latch {
                    Some(_) => None,
                    None => Some(0),
                };
                warn!("Read from CGDATA READ");
                open_bus
            }
            0x213C => {
                let vals = (self.h_latch as u16).to_le_bytes();
                let v = if self.ophct_latch {
                    (vals[1] & 0x01) | (open_bus & 0xFE)
                } else {
                    vals[0]
                };
                self.ophct_latch = !self.ophct_latch;
                v
            }
            0x213D => {
                let vals = (self.v_latch as u16).to_le_bytes();
                let v = if self.ophct_latch {
                    (vals[1] & 0x01) | (open_bus & 0xFE)
                } else {
                    vals[0]
                };
                self.ophct_latch = !self.ophct_latch;
                v
            }
            0x213E => {
                warn!("Read from PPU STAT 1");
                0
            }
            0x213F => {
                warn!("Read from PPU STAT 2");
                self.ophct_latch = false;
                self.h_latch = 0;
                self.v_latch = 0;
                u8::from(self.interlace_field) << 7
            }
            _ => {
                debug!("Unknown read PPU register {:04X}", addr);
                open_bus
            }
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
                self.forced_blanking = bit(value, 7);
                self.brightness = (value & 0x0F) as u32;
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
            0x2103 => {
                self.oam_addr = (self.oam_addr & 0x1FF) | (0x200 * (value as usize & 0x01));
                // TODO: OAM Priority bit
            }
            0x2104 => {
                if self.oam_addr % 2 == 0 {
                    self.oam_latch = value;
                } else {
                    if self.oam_addr < 0x200 {
                        // Writes only on the second write (oam_addr is odd)
                        self.write_oam_byte(self.oam_addr.wrapping_sub(1) % 0x200, self.oam_latch);
                        self.write_oam_byte(self.oam_addr % 0x200, value);
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
                self.mosaic_size = (value & 0xF0) as usize / 0x10 + 1;
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
                // Update mode 7 offsets
                let v = if value & 0x10 == 0 {
                    ((value as u16 * 0x100) + self.m7_latch as u16) as i16 & 0x1FFF
                } else {
                    (0xE000 | ((value as u16 * 0x100) + self.m7_latch as u16) as u16 & 0x1FFF)
                        as i16
                };
                if addr == 0x210D {
                    self.m7_h_off = v;
                    self.m7_latch = value;
                } else if addr == 0x210E {
                    self.m7_v_off = v;
                    self.m7_latch = value;
                }
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
                    _ => unreachable!("Invalid VRAM increment amount value: {:X}", value),
                };
                self.vram_increment_mode = match bit(value, 7) {
                    false => VramIncMode::HighReadLowWrite,
                    true => VramIncMode::LowReadHighWrite,
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
                self.write_vram(2 * remapped_addr, value);
                if self.vram_increment_mode == VramIncMode::HighReadLowWrite {
                    self.inc_vram_addr();
                }
            }
            0x2119 => {
                let remapped_addr = self.remapped_vram_addr();
                // Write the high byte
                self.write_vram(2 * remapped_addr + 1, value);
                if self.vram_increment_mode == VramIncMode::LowReadHighWrite {
                    self.inc_vram_addr();
                }
            }
            0x211A => {
                self.m7_repeat = value & 0x80 == 0;
                self.m7_fill = match value & 0x40 {
                    0x40 => Mode7Fill::Character,
                    0 => Mode7Fill::Transparent,
                    _ => unreachable!(),
                };
                self.m7_flip_h = bit(value, 0);
                self.m7_flip_v = bit(value, 1);
            }
            0x211B..=0x2120 => {
                let v = ((value as u16) << 8) | self.multi_latch as u16;
                match addr & 0xFF {
                    0x1B => self.matrix.a = v,
                    0x1C => self.matrix.b = v,
                    0x1D => self.matrix.c = v,
                    0x1E => self.matrix.d = v,
                    0x1F => {
                        // Keep it as a 13 bit signed number
                        if v & 0x1000 != 0 {
                            self.matrix.center_x = (0xE000 | (v & 0x1FFF)) as i16;
                        } else {
                            self.matrix.center_x = (v & 0x1FFF) as i16;
                        }
                    }
                    0x20 => {
                        // Keep it as a 13 bit signed number
                        if v & 0x1000 != 0 {
                            self.matrix.center_y = (0xE000 | (v & 0x1FFF)) as i16;
                        } else {
                            self.matrix.center_y = (v & 0x1FFF) as i16;
                        }
                    }
                    _ => unreachable!(),
                }
                self.multi_latch = value;
                self.refresh_multi_res();
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
            0x2125 => {
                (0..2).for_each(|i| {
                    self.windows[i].invert_sprite = bit(value, 2 * i);
                    self.windows[i].enabled_sprite = bit(value, 2 * i + 1);
                });
                (0..2).for_each(|i| {
                    self.windows[i].invert_color = bit(value, 4 + 2 * i);
                    self.windows[i].enabled_color = bit(value, 4 + 2 * i + 1);
                });
            }
            0x2126 => self.windows[0].left = value as usize,
            0x2127 => self.windows[0].right = value as usize,
            0x2128 => self.windows[1].left = value as usize,
            0x2129 => self.windows[1].right = value as usize,
            0x212A => {
                self.backgrounds.iter_mut().enumerate().for_each(|(i, b)| {
                    b.window_mask_logic = WindowMaskLogic::from(value >> (2 * i))
                })
            }
            0x212B => {
                self.color_window_logic = WindowMaskLogic::from(value >> 2);
                self.sprite_window_logic = WindowMaskLogic::from(value);
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
            // TODO: OAM sprite window enabled/disabled for main screen/subscreen
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
            0x2130 => {
                self.direct_color = bit(value, 0);
                self.color_math_src = if bit(value, 1) {
                    ColorMathSource::Subscreen
                } else {
                    ColorMathSource::Fixed
                };
                self.color_window_sub_region = WindowRegion::from(value >> 4);
                self.color_window_main_region = WindowRegion::from(value >> 6);
            }
            0x2131 => {
                self.backgrounds
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, b)| b.color_math_enable = bit(value, i));
                self.color_math_enable_obj = bit(value, 4);
                self.color_math_enable_backdrop = bit(value, 5);
                self.color_blend_mode = ColorBlendMode::from(value >> 6);
            }
            0x2132 => {
                let v = value & 0x1F;
                (0..3).for_each(|i| {
                    if bit(value, 5 + i) {
                        self.fixed_color[i] = v as u16;
                    }
                });
            }
            // Todo
            0x2133 => {
                self.overscan = bit(value, 2);
                warn!("Write to PPU SETINI {:02X}", value);
            }
            _ => debug!("Unknown PPU register: {:04X} {:02X}", addr, value),
        }
    }
    /// Refresh the multiplication result value
    fn refresh_multi_res(&mut self) {
        // Only the lower byte of B is used for the multiplication
        self.multi_res = (self.matrix.a as i16) as i32 * (self.matrix.b & 0xFF) as i32;
    }
    fn write_vram(&mut self, addr: usize, value: u8) {
        if self.can_write_vram() {
            self.vram[addr] = value;
            // Update cache
            let cache_addr = (addr / 2) * 2;
            let low = self.vram[cache_addr % self.vram.len()];
            let high = self.vram[(cache_addr + 1) % self.vram.len()];
            self.vram_cache_2bpp[addr / 2] = core::array::from_fn(|i| {
                ((low >> (7 - i)) & 0x01) + 2 * ((high >> (7 - i)) & 0x01)
            });
        }
    }
    pub fn reset_vram_cache(&mut self) {
        self.vram.chunks(2).enumerate().for_each(|(i, v)| {
            let (low, high) = (v[0], v[1]);
            self.vram_cache_2bpp[i] = core::array::from_fn(|j| {
                ((low >> (7 - j)) & 0x01) + 2 * ((high >> (7 - j)) & 0x01)
            });
        })
    }
    fn inc_vram_addr(&mut self) {
        self.vram_addr = (self.vram_addr + self.vram_increment_amount) % self.vram.len();
    }
    fn refresh_vram_latch(&mut self) {
        let vram_addr = self.remapped_vram_addr();
        self.vram_latch_low = self.read_vram_byte(2 * vram_addr);
        self.vram_latch_high = self.read_vram_byte(2 * vram_addr + 1);
    }
    fn read_vram_byte(&self, byte_addr: usize) -> u8 {
        self.vram[byte_addr % self.vram.len()]
    }
    fn remapped_vram_addr(&self) -> usize {
        let addr = match self.vram_remap {
            0 => self.vram_addr,
            1 => {
                (self.vram_addr & 0xFF00)
                    + ((self.vram_addr >> 5) & 0x07)
                    + ((self.vram_addr << 3) & 0xF8)
            }
            2 => {
                (self.vram_addr & 0xFE00)
                    + ((self.vram_addr >> 6) & 0x07)
                    + ((self.vram_addr << 3) & 0x01F8)
            }
            3 => {
                (self.vram_addr & 0xFC00)
                    + ((self.vram_addr >> 7) & 0x07)
                    + ((self.vram_addr << 3) & 0x03F8)
            }
            _ => unreachable!("Invalid VRAM REMAP value: {:X}", self.vram_remap),
        };
        addr & 0x7FFF
    }
    /// Write a single byte to OAM
    fn write_oam_byte(&mut self, addr: usize, value: u8) {
        let addr = addr % (0x220);
        if addr > 0x220 {
            debug!("Addr {:04X}", addr);
        }
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
        self.vram_cache_2bpp[(addr / 2) % self.vram_cache_2bpp.len()]
    }
    fn get_m7_background_slice(&self, x: f32, y: f32) -> BackgroundPixel {
        let pixel = if x >= 0.0 && x < 1024.0 && y >= 0.0 && y < 1024.0 {
            Some((x, y))
        } else if self.m7_repeat {
            Some((x.rem_euclid(1024.0), y.rem_euclid(1024.0)))
        } else {
            match self.m7_fill {
                Mode7Fill::Character => Some((x.rem_euclid(8.0), y.rem_euclid(8.0))),
                Mode7Fill::Transparent => None,
            }
        };
        // At this point (x,y) should both be positive values between 0-1024
        return match pixel {
            None => None,
            Some((x, y)) => {
                let (x, y) = (x.floor() as usize, y.floor() as usize);
                // Get the index of the tile we need to draw
                let tilemap_index = (x / 8) + 128 * (y / 8);
                // Tilemap bytes are only in the low bytes
                let tile_index = self.vram[2 * tilemap_index];
                // Get the index of the tile data
                // 8bpp (1 byte per pixel) * 8x8 tiles * tile_index
                let tile_addr = 8 * 8 * tile_index as usize;
                // Tile data is in the high byte
                let pixel_index = (x % 8) + 8 * (y % 8);
                let palette_byte = self.vram[2 * (tile_addr + pixel_index) + 1];
                return if palette_byte == 0 {
                    None
                } else {
                    Some((self.cgram[palette_byte as usize & 0x7F], true))
                };
            }
        };
    }
    /// Get a slice of 8 background pixels that contains the given coordinates.
    /// Also return the X offset that the given coordinate is at in the slice.
    /// i.e. if x=13 y=0, then return the second slice in the background with offset 5
    fn get_background_slice(
        &self,
        bg_index: usize,
        (x, y): (usize, usize),
        bpp: usize,
    ) -> ([BackgroundPixel; 8], usize) {
        let b = &self.backgrounds[bg_index];
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
        // This is just a slightly more optimized way to collect them without any heap allocation
        const MAX_BPP: usize = 8;
        let mut slices = [[0; 8]; MAX_BPP / 2];
        (0..(bpp as usize / 2))
            .for_each(|i| slices[i] = self.get_2bpp_slice_at(slice_addr + 16 * i));

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
                let i = if self.bg_mode == 0 { bg_index } else { 0 };
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
        let slice_values: [BackgroundPixel; 8] = core::array::from_fn(|i| {
            let v = (0..4)
                .map(|j| {
                    let s = slices[j];
                    // Shifted left by 2 since each slice will have 2 bits per pixel
                    (s[if flip_x { 7 - i } else { i }] as usize) << (2 * j)
                })
                .sum::<usize>();
            if v == 0 {
                None
            } else {
                Some((palette[v], priority))
            }
        });
        // We "skip" the first (x % 8) pixels
        // Since each byte contains data for 8 consecutive pixels
        // if the screen is scrolled over horizontally by less than 8 pixels
        // (or any amount that isn't a multiple of 8), we need to load the
        // byte and then only use some of the data it in
        // So we skip the first (x % 8) pixels by starting with that offset
        (slice_values, (x % 8))
    }
    fn extend_background_byte_buffer(&mut self, index: usize, (x, y): (usize, usize), bpp: usize) {
        // Get the data to extend the buffer with
        let (slices, offset) = self.get_background_slice(index, (x, y), bpp);
        // Extend the buffer
        let b = &mut self.backgrounds[index];
        b.pixel_buffer.extend(&slices[offset..slices.len()]);
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
    fn dot_xy(&self) -> (usize, usize) {
        // Note the visual picture starts at dot 88
        let x = (self.dot / 4).wrapping_sub(22) % PIXELS_PER_SCANLINE;
        let y = (self.dot / 4).wrapping_sub(22) / PIXELS_PER_SCANLINE;
        (x, y)
    }
    pub fn advance_master_clock(&mut self, clock: u32) {
        (0..clock).for_each(|_| {
            self.dot = (self.dot + 1) % (4 * (PIXELS_PER_SCANLINE * SCANLINES));
            if self.dot % 4 == 0 {
                let (x, y) = self.dot_xy();
                // Todo: check if this timing is correct
                if y == 0 && x == 0 {
                    debug!("VBLANK END {}", self.bg_mode);
                    self.vblank = false;
                    self.interlace_field = !self.interlace_field;
                }
                let vblank_scanline = if self.overscan { 241 } else { 225 };
                if y == vblank_scanline && x == 0 {
                    debug!("VBLANK START");
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
                    // Update mosaic latch
                    self.mosaic_v_latch = (self.mosaic_v_latch + 1) % self.mosaic_size;
                }
                if x < 256 && y < 240 {
                    let window_vals: [bool; 2] = core::array::from_fn(|i| {
                        self.windows[i].left <= x && x <= self.windows[i].right
                    });
                    let color_vals: [bool; 2] =
                        core::array::from_fn(|i| window_vals[i] ^ self.windows[i].invert_color);
                    let color_window_value = if self.windows[0].enabled_color {
                        if self.windows[1].enabled_color {
                            self.color_window_logic
                                .compute(color_vals[0], color_vals[1])
                        } else {
                            color_vals[0]
                        }
                    } else if self.windows[1].enabled_color {
                        color_vals[1]
                    } else {
                        false
                    };
                    let bg_pixels = {
                        if self.bg_mode == 7 {
                            let a = [
                                x as f32 - self.matrix.center_x as f32 + self.m7_h_off as f32,
                                y as f32 - self.matrix.center_y as f32 + self.m7_v_off as f32,
                                1.0,
                            ];
                            let mat = [
                                [
                                    convert_8p8(self.matrix.a),
                                    convert_8p8(self.matrix.b),
                                    self.matrix.center_x as f32,
                                ],
                                [
                                    convert_8p8(self.matrix.c),
                                    convert_8p8(self.matrix.d),
                                    self.matrix.center_y as f32,
                                ],
                                [0.0, 0.0, 1.0],
                            ];
                            let res: [f32; 3] = core::array::from_fn(|i| {
                                a.iter().enumerate().map(|(j, v)| mat[i][j] * *v).sum()
                            });
                            let [x, y, _] = res;
                            let slice = self.get_m7_background_slice(x, y);
                            let x: [BackgroundPixel; 4] = [slice; 4];
                            x
                        } else {
                            // Structured (background_number, bpp)
                            let backgrounds: &[(usize, usize)] = match self.bg_mode {
                                0 => &[(0, 2), (1, 2), (2, 2), (3, 2)],
                                1 => &[(0, 4), (1, 4), (2, 2)],
                                // TBA: OPT
                                2 => &[(0, 4), (1, 4)],
                                3 => &[(0, 8), (1, 4)],
                                5 => &[(0, 4), (1, 2)],
                                7 => unreachable!("Mode 7 should be custom handled"),
                                _ => todo!("Background mode {} not implemented", self.bg_mode),
                            };
                            for (i, bpp) in backgrounds.iter() {
                                if self.backgrounds[*i].pixel_buffer.is_empty() {
                                    self.extend_background_byte_buffer(*i, (x, y), *bpp);
                                }
                            }
                            let bg_pixels: [BackgroundPixel; 4] = core::array::from_fn(|i| {
                                // Should be impossible to there to be no pixels right now
                                let b = &mut self.backgrounds[i];
                                if b.main_screen_enable || b.sub_screen_enable {
                                    // Get next pixel in the buffer
                                    let v = b.pixel_buffer.pop_front().unwrap();
                                    // Use/update mosaic latch if enabled
                                    if b.mosaic {
                                        let p = &mut b.mosaic_values[x / self.mosaic_size];
                                        if self.mosaic_v_latch == 0 && x % self.mosaic_size == 0 {
                                            *p = v;
                                        }
                                        *p
                                    } else {
                                        v
                                    }
                                } else {
                                    None
                                }
                            });
                            bg_pixels
                        }
                    };
                    // Get the pixel from a background layer with a given priority, or None if the background is transparent
                    macro_rules! bg_value {
                        ($index: expr, $priority: expr) => {{
                            bg_pixels[$index]
                                .filter(|(_, p)| *p == $priority)
                                .map(|(v, _)| v)
                        }};
                    }
                    // Get a bool returning true if a background is on a given layer (i.e. main or sub screen)
                    macro_rules! bg_on_layer {
                        ($index: expr, $enabled: ident, $window_enabled: ident) => {{
                            let b = &self.backgrounds[$index];
                            if !b.$enabled {
                                false
                            } else {
                                let wv: [bool; 2] =
                                    core::array::from_fn(|i| window_vals[i] ^ b.window_invert[i]);
                                let v = if b.$window_enabled {
                                    if b.window_enabled[0] {
                                        if b.window_enabled[1] {
                                            b.window_mask_logic.compute(wv[0], wv[1])
                                        } else {
                                            wv[0]
                                        }
                                    } else if b.window_enabled[1] {
                                        wv[1]
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                // If window returns false, should return true
                                !v
                            }
                        }};
                    }
                    // Get a tuple of the background's value and whether that pixel is on the main or sub screen
                    macro_rules! bg {
                        ($index: expr, $priority: expr) => {
                            (
                                bg_value!($index, $priority),
                                bg_on_layer!($index, main_screen_enable, windows_enabled_main),
                                bg_on_layer!($index, sub_screen_enable, windows_enabled_sub),
                                self.backgrounds[$index].color_math_enable,
                            )
                        };
                    }
                    // Calculate sprite window values
                    let sprite_windows: [bool; 2] = core::array::from_fn(|i| {
                        if self.windows[i].enabled_sprite {
                            self.windows[i].invert_sprite ^ window_vals[i]
                        } else {
                            false
                        }
                    });
                    // Calculate the actual resulting sprite window vaue
                    let sw = if sprite_windows[0] {
                        if sprite_windows[1] { false } else { true }
                    } else {
                        sprite_windows[1]
                    };
                    // Get the pixel from a sprite layer with a given priority, or None
                    macro_rules! spr {
                        ($index: expr) => {
                            (
                                if sw {
                                    None
                                } else {
                                    self.oam_buffers[$index][x]
                                },
                                self.obj_main_enable,
                                self.obj_subscreen_enable,
                                // Todo: check if sprite is using palette 4-7
                                false,
                            )
                        };
                    }
                    // Format (pixel value, draw on main screen, draw on subscreen, apply color math)
                    const EMPTY: (Option<u16>, bool, bool, bool) = (None, false, false, false);
                    // The pixels at the given dot, in order from front to back
                    // Can get the first non-None pixel to draw and discard the rest (since they will be behind)
                    let in_order_pixels: &[(Option<u16>, bool, bool, bool)] = match self.bg_mode {
                        0 => &[
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
                        ],
                        1 => &[
                            if self.bg3_prio { bg!(2, true) } else { EMPTY },
                            spr!(3),
                            bg!(0, true),
                            bg!(1, true),
                            spr!(2),
                            bg!(0, false),
                            bg!(1, false),
                            spr!(1),
                            if self.bg3_prio { EMPTY } else { bg!(2, true) },
                            spr!(0),
                            bg!(2, false),
                        ],
                        2 => &[
                            spr!(3),
                            bg!(0, true),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            bg!(0, false),
                            spr!(0),
                            bg!(1, false),
                        ],
                        3 => &[
                            spr!(3),
                            bg!(0, true),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            bg!(0, false),
                            spr!(0),
                            bg!(1, false),
                        ],
                        5 => &[
                            spr!(3),
                            bg!(0, true),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            bg!(0, false),
                            spr!(0),
                            bg!(1, false),
                        ],
                        7 => &[
                            spr!(3),
                            spr!(2),
                            bg!(1, true),
                            spr!(1),
                            // BG 0 is draw regardless of prio
                            bg!(0, true),
                            bg!(0, false),
                            spr!(0),
                            bg!(2, false),
                        ],
                        _ => todo!("Background mode {} not implemented", self.bg_mode),
                    };
                    /// This macro gets a pixel if the given field is true
                    /// Used to avoid duplicate logic for getting the main screen and sub screen pixels
                    macro_rules! get_pixel {
                        ($field: tt) => {{
                            in_order_pixels
                                .iter()
                                // todo can probably combine these two lines
                                .find(|bg_pixel| bg_pixel.0.is_some() && bg_pixel.$field)
                                .map_or(None, |b| Some((b.0.unwrap(), b.3)))
                        }};
                    }
                    // Evaluate subscreen value
                    let subscreen_val = get_pixel!(2);
                    let mainscreen_val = get_pixel!(1);
                    // Can be none if if the color window makes the sub screen transparent
                    let color_math_source =
                        if self.color_window_sub_region.compute(color_window_value) {
                            None
                        } else {
                            match self.color_math_src {
                                ColorMathSource::Subscreen => {
                                    // Backdrop for the subscreen is the fixed color
                                    subscreen_val
                                        .map_or(Some(self.fixed_color_value()), |ss| Some(ss.0))
                                }
                                ColorMathSource::Fixed => Some(self.fixed_color_value()),
                            }
                        };
                    // Whether the window is masking the main layer
                    let hide_main = self.color_window_main_region.compute(color_window_value);
                    // Color math goes here
                    let p = if hide_main {
                        0
                    } else {
                        mainscreen_val
                            .map(|b| {
                                if b.1 {
                                    match color_math_source {
                                        Some(c) => self.color_blend_mode.compute(b.0, c),
                                        None => b.0,
                                    }
                                } else {
                                    b.0
                                }
                            })
                            .unwrap_or(if self.color_math_enable_backdrop {
                                match color_math_source {
                                    Some(c) => self.color_blend_mode.compute(self.cgram[0], c),
                                    None => self.cgram[0],
                                }
                            } else {
                                self.cgram[0]
                            })
                    };
                    // Set screen pixel
                    self.screen_buffer[256 * y + x] = p & 0x7FFF;
                }
            }
        })
    }
    pub fn can_write_vram(&self) -> bool {
        // self.forced_blanking || self.is_in_vblank()
        true
    }
    pub fn is_in_vblank(&self) -> bool {
        self.cursor_y() > if self.overscan { 240 } else { 225 }
    }
    pub fn is_in_hblank(&self) -> bool {
        self.cursor_x() >= 274
    }
    /// X coordinate of the cursor
    pub fn cursor_x(&self) -> usize {
        self.dot_xy().0
    }
    /// Y coordinate of the cursor.
    /// Equivalent to the scanline number
    pub fn cursor_y(&self) -> usize {
        self.dot_xy().1
    }
    fn fixed_color_value(&self) -> u16 {
        rgb_to_color(self.fixed_color)
    }
    pub fn screen_data_rgb(&self) -> [[u8; 3]; 256 * 240] {
        core::array::from_fn(|i| color_to_rgb_bytes(self.screen_buffer[i]))
    }
}
