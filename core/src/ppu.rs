use log::*;

#[derive(Default, Copy, Clone)]
struct Background {
    // 0 = 8x8, 1 = 16x16
    size: bool,
    mosaic: bool,
    num_horz_tilemaps: u32,
    num_vert_tilemaps: u32,
    tilemap_addr: usize,
    chr_addr: usize,
    h_off: u32,
    v_off: u32,
    main_screen_enable: bool,
    sub_screen_enable: bool,
}

#[derive(PartialEq, PartialOrd)]
enum VramIncMode {
    /// Increment after reading the high byte or writing the low byte
    HighReadLowWrite = 0,
    /// Increment after reading the low byte or writing the high byte
    LowReadHighWrite = 1,
}
pub struct Ppu {
    /// VBlank flag
    vblank: bool,
    forced_blanking: bool,
    brightness: u32,
    bg_mode: u32,
    bg3_prio: bool,
    backgrounds: [Background; 4],
    mosaic_size: u32,
    /// Background H off latch
    bg_h_off: u32,
    /// Background V off latch
    bg_v_off: u32,
    obj_main_enable: bool,
    obj_subscreen_enable: bool,
    vram_increment_amount: u32,
    vram_increment_mode: VramIncMode,
    vram_addr: usize,
    vram: [u8; 0x10000],
    cgram: [u16; 0x100],
    cgram_addr: usize,
    cgram_latch: Option<u8>,
}

impl Default for Ppu {
    fn default() -> Self {
        Ppu {
            vblank: false,
            forced_blanking: false,
            brightness: 4,
            bg_mode: 0,
            bg3_prio: false,
            backgrounds: [Background::default(); 4],
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
        }
    }
}

impl Ppu {
    pub fn read_byte(&mut self, addr: usize) -> u8 {
        match addr {
            0x4210 => u8::from(self.vblank) << 7,
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
                self.brightness = (value & 0x0F) as u32;
            }
            0x2105 => {
                // Copy background sizes
                (0..4).for_each(|i| {
                    self.backgrounds[i].size = bit!(i + 4) == 1;
                });
                self.bg3_prio = (value & 0x08) != 0;
                self.bg_mode = (value & 0x0F) as u32;
            }
            0x2106 => {
                (0..4).for_each(|i| {
                    self.backgrounds[i].mosaic = bit!(i) == 1;
                });
                self.mosaic_size = (value & 0xF0) as u32 / 0x10 + 1;
            }
            0x2107..=0x210A => {
                let b = &mut self.backgrounds[addr - 0x2107];
                b.num_horz_tilemaps = bit!(0) * 2;
                b.num_vert_tilemaps = bit!(1);
                b.tilemap_addr = (value & 0xFC) as usize / 0x04;
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
                self.vram_addr = (self.vram_addr & 0x00FF) | (value as usize * 0x100);
            }
            0x2118 => {
                // Write the low byte
                self.vram[2 * self.vram_addr] = value;
                if self.vram_increment_mode == VramIncMode::HighReadLowWrite {
                    self.vram_addr = (self.vram_addr + 1) % 0x8000;
                }
            }
            0x2119 => {
                // Write the high byte
                self.vram[2 * self.vram_addr + 1] = value;
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
            0x2133 => {}
            _ => debug!("Writing {:X} to {:X}, not handled", value, addr),
        }
    }
    pub fn advance_master_clock(&mut self, clock: u32) {
        self.vblank = !self.vblank;
    }
}
