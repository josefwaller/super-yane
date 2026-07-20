use egui::{Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{emulation::Emulation, ui::colors::RED_PRIMARY};

pub fn oam(ui: &mut Ui, emu: &Emulation) {
    TableBuilder::new(ui)
        .columns(Column::auto(), 11)
        .cell_layout(Layout::top_down_justified(egui::Align::Center))
        .header(15.0, |mut row| {
            macro_rules! col {
                ($name: expr) => {
                    row.col(|ui| {
                        ui.label(RichText::new($name).color(RED_PRIMARY));
                    });
                };
            }
            col!("INDEX");
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
            body.rows(15.0, emu.console.ppu().oam_sprites.len(), |mut row| {
                macro_rules! col {
                    ($name: expr) => {
                        row.col(|ui| {
                            ui.label($name);
                        });
                    };
                }
                let i = row.index();
                let s = &emu.console.ppu().oam_sprites[i];
                col!(format!("{:X}", i));
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
            });
        });
}
