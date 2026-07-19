use std::io::{BufWriter, Write};

use super_yane::utils::color_to_rgb_bytes;

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
pub fn bytes_to_rgb(
    src_bytes: &[u8],
    width_tiles: usize,
    height_tiles: usize,
    bpp: usize,
    palette: &[u16],
    out_buf: &mut [[u8; 3]],
) {
    // Inner buf will hold the index
    // TODO: Don't allocate every call
    let mut inner_buf = vec![0u8; out_buf.len()];
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

/// Write using a buffered writer
pub fn buf_write(p: &str, it: impl Iterator<Item = String>) {
    let f = std::fs::File::create(p).unwrap();
    let mut bw = BufWriter::new(f);
    it.for_each(|line| writeln!(bw, "{}", line).unwrap());
}
