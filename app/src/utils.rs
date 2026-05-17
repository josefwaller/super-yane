use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};
use super_yane::{Console, Ppu, utils::color_to_rgb_bytes};
use wdc65816::Processor;

use crate::{ConsoleData, CpuData, PpuData};

/// Render a section of VRAM as RGBA image data.
/// Always interprets VRAM as 16 tiles wide.
/// console: The console to use the VRAM and palettes of
/// num_tiles: How many tiles to render, in (x, y) format
/// bpp: THe bits per pixel
/// tile_offset: How many tiles to skip before rendering
/// palette: The index of the palette to use
/// direct_color: Whether to render with direct color or not
/// gap: How big of a gap to leave between tiles. 0 means that tiles will be tightly packed.
/// buffer: The buffer to write to. Provided so that we don't hvae to recreate an array every time
pub fn vram_to_rgba(
    console: &Console,
    num_tiles: (usize, usize),
    bpp: usize,
    tile_offset: usize,
    palette: usize,
    direct_color: bool,
    gap: usize,
    buffer: &mut [[u8; 4]],
) {
    let num_slices = match bpp {
        2 => 1,
        4 => 2,
        8 => 4,
        _ => unreachable!("Invalid VRAM BPP: {}", bpp),
    };
    let tile_size = 8 + gap;
    let image_width = num_tiles.0 * tile_size;
    // How many slices each tile needs
    let slice_step = 8 * num_slices;
    let colors_per_palette = 2usize.pow(bpp as u32);
    (0..num_tiles.0).for_each(|tile_x| {
        (0..num_tiles.1).for_each(|tile_y| {
            let tile_index = tile_offset + tile_x + 16 * tile_y;
            (0..8).for_each(|fine_y| {
                let slice = (0..num_slices)
                    .map(|i| {
                        console
                            .ppu()
                            .get_2bpp_slice(fine_y + slice_step * tile_index + 8 * i)
                    })
                    .enumerate()
                    .fold([0; 8], |acc, (j, e)| {
                        core::array::from_fn(|k| acc[k] + (e[k] << (2 * j)))
                    });
                (0..8).for_each(|fine_x| {
                    let x = tile_size * tile_x + fine_x;
                    let y = tile_size * tile_y + fine_y;
                    let s = slice[fine_x];
                    buffer[y * image_width + x] = if direct_color {
                        [(s & 0x03) << 5, (s & 0x38) << 2, (s & 0xC0), 0xFF]
                    } else {
                        if s == 0 {
                            [0x00; 4]
                        } else {
                            let c = color_to_rgb_bytes(
                                console.ppu().cgram[colors_per_palette * palette + s as usize],
                                0xF,
                            );
                            [c[0], c[1], c[2], 0xFF]
                        }
                    };
                })
            })
        });
    });
}

/// Interprets a chunk of binary data as SNES 2bpp tile date, and rewrites it into a 2BPP format
/// * `width` is the width of the output in 8x8 tiles.
/// * `height` is the height of the output in 8x8 tiles.
/// * `buffer` is the buffer that is written to. 2BPP tiles are written sequentially, so the first 8 bytes are
/// the first slice, the first 64 bytes are the first tile.
fn bytes_to_rgb_2bpp(bytes: &[u8], width_tiles: usize, height_tiles: usize, buffer: &mut [u8]) {
    let width_pixels = width_tiles * 8;
    (0..height_tiles).for_each(|tile_y| {
        (0..width_tiles).for_each(|tile_x| {
            // Render tile at (tile_x, tile_y)
            let tile_index = tile_y * width_tiles + tile_x;
            // 2 bytes per slice * 8 slices per tile
            let src_tile_address = 2 * 8 * tile_index;
            // Copy each slice
            (0..8).for_each(|y| {
                // Get low and high slice
                let low = bytes[src_tile_address + 2 * y];
                let high = bytes[src_tile_address + 2 * y + 1];
                // Get the tile (x, y) index to write to
                let dest_tile_x = tile_index % width_tiles;
                let dest_tile_y = tile_index / width_tiles;
                // Destination to write to
                let dest_tile_address =
                    8 * dest_tile_x + 8 * width_pixels * dest_tile_y + width_pixels * y;
                (0..8).for_each(|x| {
                    // Get individual pixel value
                    let val = 2 * ((low >> (7 - x)) & 0x01) + ((high >> (7 - x)) & 0x01);
                    // Write value
                    buffer[dest_tile_address + x] = val;
                });
            })
        });
    })
}

pub fn bytes_to_rgb(
    bytes: &[u8],
    width_tiles: usize,
    height_tiles: usize,
    bpp: usize,
    buffer: &mut [u8],
) {
    // First parse as 2Bpp
    // Todo: Make this not a vec
    let mut buffer_2bpp = vec![0u8; 8 * 8 * width_tiles * height_tiles * 4];
    let multi = match bpp {
        2 => 1,
        4 => 2,
        8 => 4,
        // _ => panic!("Invalid BPP provided: {}", bpp),
        _ => 1,
    };
    bytes_to_rgb_2bpp(bytes, width_tiles * multi, height_tiles, &mut buffer_2bpp);
    // Get number of 2bpp pixels per slice
    let pixels_per_slice = 8 * multi;
    // Combine the slices
    (0..(8 * height_tiles * width_tiles)).for_each(|i| {
        // Get the pixels for this slice
        let pixels = &buffer_2bpp[(pixels_per_slice * i)..(pixels_per_slice * (i + 1))];
        (0..8).for_each(|x| {
            buffer[8 * i + x] = (0..multi).map(|j| pixels[x + 8 * j] << (2 * j)).sum();
        })
    });
}

/// Shorthand for converting a byte to a 2-digit hex number
fn h8(value: u8) -> SharedString {
    format!("{:02X}", value).into()
}
/// Shorthand for converting a u16 into a 4-digit hex number
fn h16(value: impl Into<u16>) -> SharedString {
    format!("{:04X}", value.into()).into()
}
/// Shorthand for converting a bool to a 1 or 0 shared string
fn b(value: bool) -> SharedString {
    format!("{}", u8::from(value)).into()
}
impl Into<CpuData> for &Processor {
    fn into(self) -> CpuData {
        let Processor {
            a,
            b: b_reg,
            xl,
            xh,
            yl,
            yh,
            pbr,
            pc,
            dbr,
            dl,
            dh,
            s,
            p,
            ..
        } = *self;
        CpuData {
            pbr: h8(pbr),
            pc: h16(pc),
            a: h8(a),
            b: h8(b_reg),
            c: h16(self.c()),
            x: h16(self.x()),
            xl: h8(xl),
            xh: h8(xh),
            y: h16(self.y()),
            yl: h8(yl),
            yh: h8(yh),
            sp: h16(s),
            dbr: h8(dbr),
            d: h16(self.dr()),
            dl: h8(dl),
            dh: h8(dh),
            p: h8(p.to_byte(true)),
            p_z: b(p.z),
            p_v: b(p.v),
            p_n: b(p.n),
            p_c: b(p.c),
            p_d: b(p.d),
            p_i: b(p.i),
            p_m: b(p.m),
            p_e: b(p.e),
            p_xb: b(p.xb),
        }
    }
}

impl Into<PpuData> for &Ppu {
    fn into(self) -> PpuData {
        let Ppu {
            vblank,
            forced_blanking,
            brightness,
            bg_mode,
            bg3_prio,
            mosaic_size,
            vram_addr,
            vram_increment_mode,
            vram_increment_amount,
            cgram_addr,
            ..
        } = *self;
        PpuData {
            vblank: b(vblank),
            forced_blanking: b(forced_blanking),
            brightness: h8(brightness),
            bg_mode: h8(bg_mode as u8),
            bg3_prio: b(bg3_prio),
            mosaic_size: h8(mosaic_size as u8),
            vram_addr: h16(vram_addr as u16),
            vram_inc_mode: vram_increment_mode.to_string().into(),
            vram_inc_amt: h16(vram_increment_amount as u16),
            cgram_addr: h16(cgram_addr as u16),
        }
    }
}
