use egui::{
    Color32, ColorImage, ComboBox, Layout, Pos2, Rect, RichText, TextureHandle, TextureOptions, Ui,
};
use egui_extras::{Column, TableBuilder};

use crate::utils::bytes_to_rgb;

const TEXTURE_WIDTH_TILES: usize = 16;
const TEXTURE_WIDTH_PIXELS: usize = 8 * TEXTURE_WIDTH_TILES;

// WGPU enforced max texture height
const TEXTURE_HEIGHT: usize = 8192;

/// Update the texture, only updating the rows that the user has scrolled to right now.
fn update_texture(
    tile_row: usize,
    height_tiles: usize,
    data: &[u8],
    palette: &[u16],
    bpp: usize,
    texture: &mut TextureHandle,
) {
    // Due to how tiles are structured in VRAM, we need to round down to the nearest tile (i.e 8 pixels)
    let pixel_row = 8 * tile_row;
    let total_height = 8 * height_tiles;
    let bytes_per_tile = 8 * bpp;
    let offset_byte = pixel_row / 8 * TEXTURE_WIDTH_TILES * bytes_per_tile;
    // Todo: Not allocate every frame
    let mut buf = vec![[0; 3]; TEXTURE_WIDTH_PIXELS * total_height];
    // Get RGB data
    bytes_to_rgb(
        &data[offset_byte..],
        16,
        total_height / 8,
        bpp,
        palette,
        &mut buf,
    );
    // Copy only section of texture
    texture.set_partial(
        [0, pixel_row % TEXTURE_HEIGHT],
        ColorImage::from_rgb([TEXTURE_WIDTH_PIXELS, total_height], buf.as_flattened()),
        TextureOptions::NEAREST,
    );
}

#[derive(Clone)]
struct State {
    texture: Option<TextureHandle>,
    bpp_index: usize,
    palette_index: usize,
}

pub fn binary_tiles(ui: &mut Ui, data: &[u8], palette: &[u16], color: Color32) {
    let id = ui.unique_id();
    // Get state
    let mut state = ui.ctx().data_mut(|d| {
        d.get_persisted(id).unwrap_or(State {
            texture: None,
            bpp_index: 0,
            palette_index: 0,
        })
    });
    // Show BPP selector
    const BPPS: [usize; 3] = [2, 4, 8];
    ComboBox::new(id.with("bpp"), "BPP")
        .selected_text(format!("{}", state.bpp_index))
        .show_index(ui, &mut state.bpp_index, BPPS.len(), |i| {
            format!("{}", BPPS[i])
        });
    let bpp = BPPS[state.bpp_index];
    // Show palette selector
    let (num_palettes, palette_len) = match bpp {
        2 => (64, 4),
        4 => (16, 16),
        8 => (1, 256),
        _ => unimplemented!("Invalid BPP"),
    };
    ComboBox::new(id.with("palette"), "Palette")
        .selected_text(format!("{}", state.palette_index))
        .show_index(ui, &mut state.palette_index, num_palettes, |i| {
            format!("{}", i)
        });
    // Ensure palette index is not too high
    state.palette_index = state.palette_index.min(num_palettes);

    let texture_height = data.len() * 8 / bpp / TEXTURE_WIDTH_PIXELS;
    let mut texture = state.texture.clone().unwrap_or_else(|| {
        ui.load_texture(
            id.value().to_string(),
            ColorImage::filled([TEXTURE_WIDTH_PIXELS, 0], Color32::RED),
            Default::default(),
        )
    });
    state.texture = Some(texture.clone());
    // Ensure texture size is correct
    if texture.size() != [TEXTURE_WIDTH_PIXELS, TEXTURE_HEIGHT] {
        texture.set(
            ColorImage::filled([TEXTURE_WIDTH_PIXELS, TEXTURE_HEIGHT], Color32::RED),
            TextureOptions::NEAREST,
        );
    }
    ui.ctx().data_mut(|d| d.insert_persisted(id, state.clone()));
    // Rows should be square
    let row_height = (ui.available_width() / TEXTURE_WIDTH_TILES as f32).floor();
    TableBuilder::new(ui)
        .column(Column::auto())
        .columns(Column::remainder(), TEXTURE_WIDTH_TILES)
        .cell_layout(Layout::centered_and_justified(egui::Direction::LeftToRight))
        .header(18.0, |mut row| {
            row.col(|ui| {
                ui.label("IDX/ADDR");
            });
            for index in 0..TEXTURE_WIDTH_TILES {
                row.col(|ui| {
                    ui.label(RichText::new(format!("+{:X}", index)).color(color));
                });
            }
        })
        // Each row is one tile (8 pixels)
        .body(|body| {
            // Update only the section of the texture we are looking at
            body.rows(row_height, texture_height / 8, |mut row| {
                let index = row.index();
                // Update row texture
                update_texture(
                    index,
                    1,
                    data,
                    &palette[(palette_len * state.palette_index)..],
                    bpp,
                    &mut texture,
                );
                row.col(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{:X}/{:04X}",
                            TEXTURE_WIDTH_TILES * index,
                            TEXTURE_WIDTH_TILES * index * 8 * bpp
                        ))
                        .color(color),
                    );
                });
                let img_width = 1.0 / TEXTURE_WIDTH_TILES as f32;
                let img_height = 1.0 / (TEXTURE_HEIGHT / 8) as f32;
                let texture_index = index % (TEXTURE_HEIGHT / 8);
                for x in 0..TEXTURE_WIDTH_TILES {
                    row.col(|ui| {
                        ui.painter().image(
                            texture.id(),
                            ui.available_rect_before_wrap(),
                            Rect::from_min_max(
                                Pos2::new(x as f32 * img_width, texture_index as f32 * img_height),
                                Pos2::new(
                                    (x + 1) as f32 * img_width,
                                    (texture_index + 1) as f32 * img_height,
                                ),
                            ),
                            Color32::WHITE,
                        );
                    });
                }
            })
        });
}
