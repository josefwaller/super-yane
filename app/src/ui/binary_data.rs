use egui::{
    AtomExt, Color32, ColorImage, Image, ScrollArea, TextureHandle, TextureOptions, Ui, Vec2,
};

use crate::{ui::binary_table::binary_table, utils::bytes_to_rgb};

#[derive(Clone)]
struct State {
    view_tiles: bool,
    texture: Option<TextureHandle>,
}

const TEXTURE_WIDTH_TILES: usize = 16;
const TEXTURE_WIDTH_PIXELS: usize = 8 * TEXTURE_WIDTH_TILES;

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
            log::debug!("LOAD TEXTURE");
            ui.load_texture(
                id.value().to_string(),
                ColorImage::filled([TEXTURE_WIDTH_PIXELS, 0], Color32::RED),
                Default::default(),
            )
        });
        state.texture = Some(texture.clone());
        // Ensure texture size is correct
        if texture.size() != [TEXTURE_WIDTH_PIXELS, texture_height] {
            log::debug!("CHANGE SIZE");
            texture.set(
                ColorImage::filled([TEXTURE_WIDTH_PIXELS, texture_height], Color32::RED),
                TextureOptions::NEAREST,
            );
        }
        ui.ctx().data_mut(|d| d.insert_persisted(id, state.clone()));
        // Show data
        if state.view_tiles {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_viewport(ui, |ui, viewport| {
                    // Render screen data onto texture
                    let screen_pixel_per_pixel = viewport.width() / TEXTURE_WIDTH_PIXELS as f32;
                    // Due to how tiles are structured in VRAM, we need to round down to the nearest tile (i.e 8 pixels)
                    let offset_row =
                        ((viewport.top() / screen_pixel_per_pixel).floor() as usize / 8) * 8;
                    let height =
                        ((viewport.height() / screen_pixel_per_pixel).ceil() as usize / 8 + 2) * 8;
                    // Don't go past the last row
                    let offset_row = offset_row.min(texture_height.saturating_sub(height));
                    let bytes_per_tile = 8 * bpp;
                    let offset_byte = offset_row / 8 * TEXTURE_WIDTH_TILES * bytes_per_tile;
                    let mut buf = vec![[0; 3]; TEXTURE_WIDTH_PIXELS * height];
                    bytes_to_rgb(&data[offset_byte..], 16, height / 8, bpp, palette, &mut buf);
                    texture.set_partial(
                        [0, offset_row],
                        ColorImage::from_rgb([TEXTURE_WIDTH_PIXELS, height], buf.as_flattened()),
                        TextureOptions::NEAREST,
                    );
                    let img = Image::from_texture((texture.id(), texture.size_vec2()))
                        .fit_to_exact_size(Vec2 {
                            x: ui.available_width(),
                            y: f32::INFINITY,
                        })
                        .max_size(Vec2::INFINITY);
                    ui.add(img);
                });
        } else {
            binary_table(ui, data, color);
        }
    });
}
