use std::{error::Error, rc::Rc};

use log::debug;
use slint::{
    Image, Model, ModelExt, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, VecModel,
};
use super_yane::{
    Background, Console, InputPort, Ppu,
    apu::{Apu, Dsp, Voice},
    ppu::Sprite,
    utils::{color_to_rgb, color_to_rgb_bytes},
};
use wdc65816::{Processor, StatusRegister};

pub enum BinaryDataSrc {
    Wram,
    Vram,
    Cgram,
    Aram,
    Cartridge,
}
/// Interprets a chunk of binary data as SNES 2bpp tile date, and rewrites it into a 2BPP format
/// * `width` is the width of the output in 8x8 tiles.
/// * `height` is the height of the output in 8x8 tiles.
/// * `buffer` is the buffer that is written to. 2BPP tiles are written sequentially, so the first 8 bytes are
/// the first slice, the first 64 bytes are the first tile.
fn bytes_to_2bpp_index(bytes: &[u8], width_tiles: usize, height_tiles: usize, buffer: &mut [u8]) {
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
/// Convert tile data bytes to an array where each entry represents the index of the color in the palette
pub fn bytes_to_index(
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
    bytes_to_2bpp_index(bytes, width_tiles * multi, height_tiles, &mut buffer_2bpp);
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
/// Convert a chunk of tile data to an RGB 2D array
pub fn bytes_to_rgb<const W: usize>(
    src_bytes: &[u8],
    width_tiles: usize,
    height_tiles: usize,
    bpp: usize,
    palette: &[u16],
    out_buf: &mut [[u8; 3]; W],
) {
    // Inner buf will hold the index
    let mut inner_buf = [0u8; W];
    bytes_to_index(src_bytes, width_tiles, height_tiles, bpp, &mut inner_buf);
    const BRIGHTNESS: u8 = 0x0F;
    // Convert to RGB
    inner_buf.iter().enumerate().for_each(|(index, value)| {
        out_buf[index] = if *value == 0 {
            [0; 3]
        } else {
            color_to_rgb_bytes(palette[*value as usize], BRIGHTNESS)
        };
    });
}
const DATA_WIDTH: usize = 32;
const DATA_HEIGHT: usize = 8;
// pub fn update_binary_data(
//     c: &Console,
//     offset: usize,
//     ram_type: BinaryDataSrc,
//     bpp: i32,
//     palette_index: usize,
//     ui: &AppWindow,
// ) {
//     // Initialize data if empty
//     if ui.get_binary_data().row_count() < DATA_HEIGHT {
//         ui.set_binary_data(ModelRc::from(Rc::from(VecModel::from_iter(
//             (0..DATA_HEIGHT)
//                 .map(|_| ModelRc::from(Rc::from(VecModel::from_iter((0..DATA_WIDTH).map(|_| 0))))),
//         ))));
//     }
//     // Create a copy of CGRAM as a u8 array
//     let cgram_arr: [u8; 0x200] =
//         core::array::from_fn(|i| c.ppu().cgram[i / 2].to_le_bytes()[i % 2]);
//     // Get data as slice
//     use BinaryDataSrc::*;
//     let (data_src, data_len): (&[u8], usize) = match ram_type {
//         Vram => (&c.ppu().vram, c.ppu().vram.len()),
//         Cgram => (&cgram_arr, 2 * c.ppu().cgram.len()),
//         Wram => (c.ram().as_slice(), c.ram().len()),
//         Aram => (c.apu().ram(), c.apu().ram().len()),
//         Cartridge => (&c.cartridge().data, c.cartridge().data.len()),
//     };
//     // Copy binary data
//     let mut it = data_src.iter().skip(offset);
//     (0..DATA_HEIGHT).for_each(|i| {
//         (0..DATA_WIDTH).for_each(|j| {
//             ui.get_binary_data()
//                 .row_data_tracked(i)
//                 .unwrap()
//                 .set_row_data(j, it.next().unwrap_or(&0).clone() as i32)
//         })
//     });
//     ui.set_binary_data_len(data_len as i32);
//     // Collect colors
//     let colors: [[u8; 3]; 256] =
//         core::array::from_fn(|i| color_to_rgb_bytes(c.ppu().cgram[i], 0xF));
//     let palette_size = match bpp {
//         2 => 4,
//         4 => 16,
//         8 => 64,
//         _ => 4,
//     };
//     let palette = &colors[palette_index as usize * palette_size..];
//     // Map data to 2BPP tile
//     const NUM_TILES_WIDTH: usize = 16;
//     const NUM_TILES_HEIGHT: usize = 4;
//     let mut buffer = [0u8; 8 * 8 * NUM_TILES_WIDTH * NUM_TILES_HEIGHT];
//     // Copy data to image buffer
//     bytes_to_index(
//         &data_src[offset..],
//         NUM_TILES_WIDTH,
//         NUM_TILES_HEIGHT,
//         bpp as usize,
//         &mut buffer,
//     );
//     // Map data to RGB
//     let rgb_data: [[u8; 3]; 8 * 8 * NUM_TILES_WIDTH * NUM_TILES_HEIGHT] =
//         core::array::from_fn(|i| palette[buffer[i] as usize]);
//     // Copy to slint buffer
//     let mut buf = if ui.get_binary_image().size().width == 0 {
//         SharedPixelBuffer::new(8 * NUM_TILES_WIDTH as u32, 8 * NUM_TILES_HEIGHT as u32)
//     } else {
//         ui.get_binary_image().to_rgb8().unwrap()
//     };
//     buf.make_mut_bytes()
//         .copy_from_slice(rgb_data.as_flattened());
//     ui.set_binary_image(Image::from_rgb8(buf));
// }
