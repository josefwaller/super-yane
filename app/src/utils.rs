use std::rc::Rc;

use slint::{ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, VecModel};
use super_yane::{Console, Ppu, utils::color_to_rgb_bytes};
use wdc65816::{Processor, StatusRegister};

use crate::{BinaryDataSrc, ConsoleData, CpuData, PpuData, StatusRegisterData};

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
                let high = bytes.get(src_tile_address + 2 * y).unwrap_or(&0);
                let low = bytes.get(src_tile_address + 2 * y + 1).unwrap_or(&0);
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
pub fn get_binary_data(
    c: &Console,
    offset: usize,
    ram_type: BinaryDataSrc,
    bpp: i32,
    palette_index: usize,
) -> (ModelRc<ModelRc<i32>>, SharedPixelBuffer<Rgb8Pixel>, usize) {
    // Copy some section of ram
    let mut data = [[0u8; 32]; 8];
    // Create a copy of CGRAM as a u8 array
    let cgram_arr: [u8; 0x200] =
        core::array::from_fn(|i| c.ppu().cgram[i / 2].to_le_bytes()[i % 2]);
    // Get data as slice
    use BinaryDataSrc::*;
    let (data_src, data_len): (&[u8], usize) = match ram_type {
        Vram => (&c.ppu().vram, c.ppu().vram.len()),
        Cgram => (&cgram_arr, 2 * c.ppu().cgram.len()),
        Wram => (c.ram().as_slice(), c.ram().len()),
        Cartridge => (&c.cartridge().data, c.cartridge().data.len()),
    };
    // Copy binary data to array
    let mut it = data_src.iter().skip(offset);
    (0..8).for_each(|i| (0..32).for_each(|j| data[i][j] = it.next().unwrap_or(&0).clone()));
    // Collect colors
    let colors: [[u8; 3]; 256] =
        core::array::from_fn(|i| color_to_rgb_bytes(c.ppu().cgram[i], 0xF));
    let palette_size = match bpp {
        2 => 4,
        4 => 16,
        8 => 64,
        _ => 4,
    };
    let palette = &colors[palette_index as usize * palette_size..];
    // Map data to 2BPP tile
    const NUM_TILES_WIDTH: usize = 16;
    const NUM_TILES_HEIGHT: usize = 4;
    let mut buffer = [0u8; 8 * 8 * NUM_TILES_WIDTH * NUM_TILES_HEIGHT];
    // Copy data to image buffer
    bytes_to_rgb(
        &data_src[offset..],
        NUM_TILES_WIDTH,
        NUM_TILES_HEIGHT,
        bpp as usize,
        &mut buffer,
    );
    // Map data to RGB
    let rgb_data: [[u8; 3]; 8 * 8 * NUM_TILES_WIDTH * NUM_TILES_HEIGHT] =
        core::array::from_fn(|i| palette[buffer[i] as usize]);
    let buf = SharedPixelBuffer::clone_from_slice(
        rgb_data.as_flattened(),
        8 * NUM_TILES_WIDTH as u32,
        8 * NUM_TILES_HEIGHT as u32,
    );
    return (
        ModelRc::from(Rc::from(VecModel::from_iter((0..8).map(|i| {
            ModelRc::from(Rc::from(VecModel::from_iter(
                (0..32).map(|j| data[i][j] as i32),
            )))
        })))),
        buf,
        data_len,
    );
}
// Macro to copy a bunch of fields between structs
macro_rules! copy_fields {
    ($from: ident, $to: ident, $($field:ident),*) => {
        $(
            $to.$field = $from.$field.into();
        )*
    };
}
// Macro to copy a bunch of integer fields between structs
macro_rules! copy_int_fields {
    ($from: ident, $to: ident, $($field:ident),*) => {
        $(
            $to.$field = ($from.$field as i32).into();
        )*
    };
}

impl Into<StatusRegisterData> for &StatusRegister {
    fn into(self) -> StatusRegisterData {
        let mut data = StatusRegisterData::default();
        data.value = self.to_byte(false) as i32;
        copy_fields!(self, data, c, z, n, d, i, m, v, e, xb);
        data
    }
}

impl Into<CpuData> for &Processor {
    fn into(self) -> CpuData {
        let mut data = CpuData::default();
        copy_int_fields!(self, data, pc, pbr, a, b, yl, yh, dl, dh, dbr, s);
        data.p = (&self.p).into();
        data
    }
}

impl Into<PpuData> for &Ppu {
    fn into(self) -> PpuData {
        let mut data = PpuData::default();
        copy_int_fields!(
            self,
            data,
            vblank,
            forced_blanking,
            brightness,
            bg_mode,
            bg3_prio,
            mosaic_size,
            vram_addr,
            cgram_addr
        );
        data.vram_increment_mode = format!("{}", self.vram_increment_mode).into();
        data
    }
}
