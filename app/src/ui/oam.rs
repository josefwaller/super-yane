use egui::{
    Color32, ColorImage, Label, Layout, Pos2, Rect, RichText, TextWrapMode, TextureHandle,
    TextureOptions, Ui,
};
use egui_extras::{Column, TableBuilder};
use super_yane::{Console, ppu::NUM_SPRITES};

use crate::{emulation::Emulation, ui::colors::RED_PRIMARY, utils::bytes_to_rgb};

const MAX_SPRITE_SIZE: usize = 64;
pub const TEXTURE_WIDTH: usize = MAX_SPRITE_SIZE;
pub const TEXTURE_HEIGHT: usize = NUM_SPRITES * MAX_SPRITE_SIZE;

const SPRITE_COLUMN_SIZE: f32 = 64.0;

#[derive(Clone)]
struct State {
    texture: Option<TextureHandle>,
}

fn update_texture(texture: &mut TextureHandle, console: &Console, index: usize) {
    let s = &console.ppu().oam_sprites[index];
    let size = console.ppu().oam_sizes[s.size_select];
    let mut buf = [[0u8; 3]; MAX_SPRITE_SIZE * MAX_SPRITE_SIZE];
    // Get RGB data
    for y in 0..(size.1 / 8) {
        bytes_to_rgb(
            &console.ppu().vram[console.ppu().sprite_tile_slice_addr(s, y)..],
            size.0 / 8,
            1,
            4,
            &console.ppu().cgram[s.palette_addr()..],
            &mut buf[0..(size.0 * 8)],
        );
        // Copy only section of texture
        texture.set_partial(
            [0, MAX_SPRITE_SIZE * index + 8 * y],
            ColorImage::from_rgb([size.0, 8], buf[0..(size.0 * 8)].as_flattened()),
            TextureOptions::NEAREST,
        );
    }
}

pub fn oam(ui: &mut Ui, emu: &Emulation) {
    let id = ui.unique_id();
    let mut state = ui.ctx().data_mut(|data| {
        data.get_persisted(id.with("state"))
            .unwrap_or(State { texture: None })
    });
    let mut texture = state
        .texture
        .get_or_insert(ui.load_texture(
            "oam_sprite_data",
            ColorImage::filled([TEXTURE_WIDTH, TEXTURE_HEIGHT], Color32::RED),
            Default::default(),
        ))
        .clone();
    ui.ctx()
        .data_mut(|data| data.insert_persisted(id.with("state"), state));

    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::auto().at_least(SPRITE_COLUMN_SIZE))
        .columns(Column::auto(), 9)
        .cell_layout(Layout::top_down_justified(egui::Align::Center))
        .header(15.0, |mut row| {
            macro_rules! col {
                ($name: expr) => {
                    row.col(|ui| {
                        ui.add(
                            Label::new(RichText::new($name).color(RED_PRIMARY))
                                .wrap_mode(TextWrapMode::Extend),
                        );
                    });
                };
            }
            col!("INDEX");
            col!("TILE");
            col!("X");
            col!("Y");
            col!("TILE INDEX");
            col!("TILE ADDR");
            col!("FLIP X");
            col!("FLIP Y");
            col!("PRIORITY");
            col!("PALETTE");
            col!("SIZE");
        })
        .body(|body| {
            body.rows(
                SPRITE_COLUMN_SIZE,
                emu.console.ppu().oam_sprites.len(),
                |mut row| {
                    macro_rules! col {
                        ($name: expr) => {
                            row.col(|ui| {
                                ui.label($name);
                            });
                        };
                    }
                    let i = row.index();
                    let s = &emu.console.ppu().oam_sprites[i];
                    let size = emu.console.ppu().oam_sizes[s.size_select];
                    col!(format!("{:X}", i));
                    row.col(|ui| {
                        let tile_width = size.0 as f32 / TEXTURE_WIDTH as f32;
                        let tile_height = size.1 as f32 / TEXTURE_HEIGHT as f32;
                        let max_tile_height = MAX_SPRITE_SIZE as f32 / TEXTURE_HEIGHT as f32;
                        update_texture(&mut texture, &emu.console, i);
                        let mut size = ui.available_rect_before_wrap();
                        size.set_width(SPRITE_COLUMN_SIZE);
                        ui.painter().image(
                            texture.id(),
                            size,
                            Rect::from_min_max(
                                Pos2::new(0.0, i as f32 * max_tile_height),
                                Pos2::new(tile_width, i as f32 * max_tile_height + tile_height),
                            ),
                            Color32::WHITE,
                        );
                    });
                    col!(format!("{:02X}", s.x));
                    col!(format!("{:02X}", s.y));
                    col!(format!("{:02X}", s.tile_index()));
                    col!(format!(
                        "{:02X}",
                        emu.console.ppu().sprite_tile_slice_addr(s, 0)
                    ));
                    col!(format!("{}", s.flip_x));
                    col!(format!("{}", s.flip_y));
                    col!(format!("{:02X}", s.priority));
                    col!(format!("{:02X}", s.palette_index));
                    col!(format!(
                        "{:02X?}",
                        emu.console.ppu().oam_sizes[s.size_select]
                    ));
                },
            );
        });
}
