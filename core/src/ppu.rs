#[derive(Default)]
struct Background {
    // 0 = 8x8, 1 = 16x16
    size: bool,
    mosaic: bool,
    num_horz_tilemaps: u32,
    num_vert_tilemaps: u32,
    tilemap_addr: u32,
    chr_addr: u32,
    h_off: u32,
    v_off: u32,
    main_screen_enable: bool,
    sub_screen_enable: bool,
}
#[derive(Default)]
pub struct Ppu {
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
    vram_increment_mode: bool,
}

impl Ppu {
    pub fn read_byte(&mut self, addr: usize) -> u8 {
        0
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
                self.mosaic_size = (value & 0xF0) as u32 / 0x10;
            }
            0x2107..=0x210A => {
                let b = &mut self.backgrounds[addr - 0x2107];
                b.num_horz_tilemaps = bit!(0) * 2;
                b.num_vert_tilemaps = bit!(1);
                b.tilemap_addr = (value & 0xFC) as u32 / 0x04;
            }
            0x210B => {
                self.backgrounds[0].chr_addr = (value as u32 & 0x0F) << 12;
                self.backgrounds[1].chr_addr = (value as u32 & 0xF0) << (12 - 4);
            }
            0x210C => {
                self.backgrounds[2].chr_addr = (value as u32 & 0x0F) << 12;
                self.backgrounds[3].chr_addr = (value as u32 & 0xF0) << (12 - 4);
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
                self.vram_increment_mode = bit!(7) == 1;
            }
            0x2121 => {}
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
            _ => panic!(
                "Unexpected address passed to Ppu::write_byte: {:X} (writing {:X})",
                addr, value
            ),
        }
    }
}
