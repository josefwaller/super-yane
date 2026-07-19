use egui::{
    AtomExt, Color32, ColorImage, Image, Layout, Pos2, Rect, RichText, ScrollArea, TextureHandle,
    TextureOptions, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};

use crate::{
    ui::{binary_table::binary_table, binary_tiles},
    utils::bytes_to_rgb,
};

#[derive(Clone)]
struct State {
    view_tiles: bool,
}

pub fn binary_data(ui: &mut Ui, data: &[u8], color: Color32, palette: &[u16]) {
    ui.vertical(|ui| {
        let id = ui.unique_id();
        // Show view tiles checkbox
        let mut state = ui
            .ctx()
            .data_mut(|d| d.get_persisted(id).unwrap_or(State { view_tiles: false }));
        ui.checkbox(&mut state.view_tiles, "View as tiles");
        ui.ctx().data_mut(|d| d.insert_persisted(id, state.clone()));
        // Show data
        if state.view_tiles {
            binary_tiles(ui, data, palette, color);
        } else {
            binary_table(ui, data, color);
        }
    });
}
