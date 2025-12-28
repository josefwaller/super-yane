use super_yane::{Console, utils::color_to_rgb_bytes};

pub mod utils {
    macro_rules! hex_fmt {
        () => {
            "0x{:04X}"
        };
    }
    macro_rules! table_row {
        ($label: expr, $field: expr, $format_str: expr) => {
            ($label, text(format!($format_str, $field)).into())
        };
        ($label: expr, $field: ident) => {
            ppu_val!($label, $field, "{}")
        };
    }
    pub(crate) use hex_fmt;
    pub(crate) use table_row;
}
/// Render a section of VRAM as RGBA image data
/// console: The console to use the VRAM and palettes of
/// num_tiles: How many tiles to render, in (x, y) format
/// bpp: THe bits per pixel
/// tile_offset: How many tiles to skip before rendering
/// palette: The index of the palette to use
/// direct_color: Whether to render with direct color or not
/// buffer: The buffer to write to. Provided so that we don't hvae to recreate an array every time
pub fn vram_to_rgba(
    console: &Console,
    num_tiles: (usize, usize),
    bpp: usize,
    tile_offset: usize,
    palette: usize,
    direct_color: bool,
    buffer: &mut [[u8; 4]],
) {
    let num_slices = match bpp {
        2 => 1,
        4 => 2,
        8 => 4,
        _ => unreachable!("Invalid VRAM BPP: {}", bpp),
    };
    let image_width = num_tiles.0 * 8;
    // How many slices each tile needs
    let slice_step = 8 * num_slices;
    (0..num_tiles.0).for_each(|tile_x| {
        (0..num_tiles.1).for_each(|tile_y| {
            let tile_index = tile_offset + tile_x + num_tiles.0 * tile_y;
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
                (0..8).for_each(|i| {
                    buffer[8 * tile_x + 8 * image_width * tile_y + image_width * fine_y + i] =
                        if direct_color {
                            [
                                (slice[i] & 0x03) << 5,
                                (slice[i] & 0x38) << 2,
                                (slice[i] & 0xC0),
                                0xFF,
                            ]
                        } else {
                            if i == 0 {
                                [0x00; 4]
                            } else {
                                let colors_per_palette = 2usize.pow(bpp as u32);
                                let c = color_to_rgb_bytes(
                                    console.ppu().cgram
                                        [colors_per_palette * palette + slice[i] as usize],
                                );
                                [c[0], c[1], c[2], 0xFF]
                            }
                        };
                })
            })
        });
    });
}
