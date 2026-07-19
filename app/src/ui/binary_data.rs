use egui::{
    AtomExt, Color32, ColorImage, Image, Layout, Pos2, Rect, RichText, ScrollArea, TextureHandle,
    TextureOptions, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};

use crate::{ui::binary_table::binary_table, utils::bytes_to_rgb};

#[derive(Clone)]
struct State {
    view_tiles: bool,
    texture: Option<TextureHandle>,
}

const TEXTURE_WIDTH_TILES: usize = 16;
const TEXTURE_WIDTH_PIXELS: usize = 8 * TEXTURE_WIDTH_TILES;

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
        [0, pixel_row],
        ColorImage::from_rgb([TEXTURE_WIDTH_PIXELS, total_height], buf.as_flattened()),
        TextureOptions::NEAREST,
    );
}

pub fn binary_data(ui: &mut Ui, data: &[u8], color: Color32, palette: &[u16]) {
    ui.vertical(|ui| {
        let id = ui.unique_id();
        let bpp = 4;
        let texture_height = data.len() * 8 / bpp / TEXTURE_WIDTH_PIXELS;
        // Show view tiles checkbox
        let mut state = ui.ctx().data_mut(|d| {
            d.get_persisted(id).unwrap_or(State {
                view_tiles: false,
                texture: None,
            })
        });
        ui.checkbox(&mut state.view_tiles, "View as tiles");
        let mut texture = state.texture.clone().unwrap_or_else(|| {
            ui.load_texture(
                id.value().to_string(),
                ColorImage::filled([TEXTURE_WIDTH_PIXELS, 0], Color32::RED),
                Default::default(),
            )
        });
        state.texture = Some(texture.clone());
        // Ensure texture size is correct
        if texture.size() != [TEXTURE_WIDTH_PIXELS, texture_height] {
            texture.set(
                ColorImage::filled([TEXTURE_WIDTH_PIXELS, texture_height], Color32::RED),
                TextureOptions::NEAREST,
            );
        }
        ui.ctx().data_mut(|d| d.insert_persisted(id, state.clone()));
        // Show data
        if state.view_tiles {
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
                        update_texture(index, 1, data, palette, bpp, &mut texture);
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
                        let img_height = 1.0 / (texture_height / 8) as f32;
                        for x in 0..TEXTURE_WIDTH_TILES {
                            row.col(|ui| {
                                ui.painter().image(
                                    texture.id(),
                                    ui.available_rect_before_wrap(),
                                    Rect::from_min_max(
                                        Pos2::new(x as f32 * img_width, index as f32 * img_height),
                                        Pos2::new(
                                            (x + 1) as f32 * img_width,
                                            (index + 1) as f32 * img_height,
                                        ),
                                    ),
                                    Color32::WHITE,
                                );
                            });
                        }
                    })
                });
        } else {
            binary_table(ui, data, color);
        }
    });
}
