use egui::{
    Align, Atom, Button, Color32, ColorImage, ComboBox, Image, LayerId, Layout, Pos2, Rect,
    RichText, Sense, TextureHandle, TextureOptions, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};
use super_yane::utils::color_to_rgb_bytes;

use crate::utils::{bytes_to_rgb, bytes_to_rgb_mode7};

const TEXTURE_WIDTH_TILES: usize = 16;
const TEXTURE_WIDTH_PIXELS: usize = 8 * TEXTURE_WIDTH_TILES;

// WGPU enforced max texture height
const TEXTURE_HEIGHT: usize = 8192;

#[derive(Copy, Clone)]
pub enum TileDataFormat {
    Bpp(usize),
    Mode7,
}

impl ToString for TileDataFormat {
    fn to_string(&self) -> String {
        match self {
            TileDataFormat::Bpp(bpp) => format!("{}bpp", bpp),
            TileDataFormat::Mode7 => "Mode 7".to_string(),
        }
    }
}

const FORMATS: [TileDataFormat; 4] = {
    use TileDataFormat::*;
    [Bpp(2), Bpp(4), Bpp(8), Mode7]
};

/// Update the texture, only updating the rows that the user has scrolled to right now.
fn update_texture(
    tile_row: usize,
    height_tiles: usize,
    data: &[u8],
    palette: &[u16],
    format: TileDataFormat,
    direct_color: bool,
    texture: &mut TextureHandle,
) {
    // Due to how tiles are structured in VRAM, we need to round down to the nearest tile (i.e 8 pixels)
    let pixel_row = 8 * tile_row;
    let total_height = 8 * height_tiles;
    let bytes_per_tile = match format {
        TileDataFormat::Bpp(bpp) => 8 * bpp,
        TileDataFormat::Mode7 => 2 * 8 * 8,
    };
    let offset_byte = pixel_row / 8 * TEXTURE_WIDTH_TILES * bytes_per_tile;
    // Todo: Not allocate every frame
    let mut buf = vec![[0; 3]; TEXTURE_WIDTH_PIXELS * total_height];
    // Get RGB data
    match format {
        TileDataFormat::Bpp(bpp) => {
            bytes_to_rgb(
                &data[offset_byte..],
                16,
                total_height / 8,
                bpp,
                palette,
                direct_color,
                &mut buf,
            );
        }
        TileDataFormat::Mode7 => bytes_to_rgb_mode7(
            &data[offset_byte..],
            16,
            total_height / 8,
            palette,
            direct_color,
            &mut buf,
        ),
    }
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
    fmt_index: usize,
    palette_index: usize,
    direct_color: bool,
}

pub fn binary_tiles(ui: &mut Ui, data: &[u8], palette: &[u16], color: Color32) {
    let id = ui.unique_id();
    // Get state
    let mut state = ui.ctx().data_mut(|d| {
        d.get_persisted(id).unwrap_or(State {
            texture: None,
            fmt_index: 0,
            palette_index: 0,
            direct_color: false,
        })
    });
    // Show BPP selector
    ComboBox::new(id.with("bpp"), "BPP")
        .selected_text(format!("{}", state.fmt_index))
        .show_index(ui, &mut state.fmt_index, FORMATS.len(), |i| {
            format!("{}", FORMATS[i].to_string())
        });
    let format = FORMATS[state.fmt_index];
    // Show palette selector
    let (num_palettes, palette_len) = match format {
        TileDataFormat::Bpp(bpp) => match bpp {
            2 => (64, 4),
            4 => (16, 16),
            8 => (1, 256),
            _ => unimplemented!("Invalid BPP"),
        },
        TileDataFormat::Mode7 => (1, 256),
    };
    // Ensure palette index is not too high
    state.palette_index = state.palette_index.min(num_palettes - 1);
    // The actual index to drag, may be changed if we're hovering over an option
    let mut palette_index_override = state.palette_index;
    ComboBox::from_id_salt(id.with(format.to_string()).with("palette"))
        .selected_text(format!("Palette {}", state.palette_index))
        .show_ui(ui, |ui| {
            for i in 0..num_palettes {
                // Create atom for palette colors
                let atom_id = ui.unique_id().with("palette").with(i);
                let atom = Atom::custom(atom_id, Vec2::new(16.0 * palette_len as f32, 16.0));
                // Create button
                let button =
                    Button::selectable(i == state.palette_index, (format!("Palette {} ", i), atom));
                // Add button to UI
                let response = button.atom_ui(ui);
                if response.clicked() {
                    state.palette_index = i;
                }
                if response.hovered() {
                    palette_index_override = i;
                }
                // Draw palette if button is visible
                if let Some(rect) = response.rect(atom_id) {
                    let palette = &palette[(i * palette_len)..];
                    let painter = ui.painter_at(rect);
                    // Draw rect for every color
                    for j in 0..palette_len {
                        let color = color_to_rgb_bytes(palette[j], 0xF);
                        let color = Color32::from_rgb(color[0], color[1], color[2]);
                        painter.rect_filled(
                            rect.with_min_x(rect.min.x + 16.0 * j as f32)
                                .with_max_x(rect.min.x + 16.0 * (j + 1) as f32),
                            0,
                            color,
                        );
                    }
                }
            }
        });

    ui.checkbox(&mut state.direct_color, "Direct Color");

    let (texture_height, bytes_per_tile) = match format {
        TileDataFormat::Bpp(bpp) => (data.len() * 8 / bpp / TEXTURE_WIDTH_PIXELS, 8 * bpp),
        TileDataFormat::Mode7 => (data.len() / 2 / (2 * 8 * 8), 2 * 8 * 8),
    };
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
                    &palette[(palette_len * palette_index_override)..],
                    format,
                    state.direct_color,
                    &mut texture,
                );
                row.col(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{:X}/{:04X}",
                            TEXTURE_WIDTH_TILES * index,
                            TEXTURE_WIDTH_TILES * index * bytes_per_tile
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
