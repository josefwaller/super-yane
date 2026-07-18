use egui::{Align, Color32, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::ui::colors::{LIGHT_GREY, WHITE};

pub fn binary_table(ui: &mut Ui, data: &[u8], color: Color32) {
    const NUM_COLS: usize = 32;
    TableBuilder::new(ui)
        .column(Column::auto())
        .columns(Column::remainder(), NUM_COLS)
        .cell_layout(Layout::right_to_left(Align::Center))
        .header(15.0, |mut row| {
            row.col(|_| {});
            for x in 0..NUM_COLS {
                row.col(|ui| {
                    ui.label(RichText::new(format!("+{:02X}", x)).color(color));
                });
            }
        })
        .body(|body| {
            body.rows(10.0, data.len() / NUM_COLS, |mut row| {
                let address = NUM_COLS * row.index();
                row.col(|ui| {
                    ui.label(RichText::new(format!("{:06X}", address)).color(color));
                });
                for x in 0..NUM_COLS {
                    let index = address + x;
                    let value = if index < data.len() {
                        Some(data[index])
                    } else {
                        None
                    };
                    row.col(|ui| {
                        ui.label(
                            RichText::new(
                                value.map(|v| format!("{:02X}", v)).unwrap_or("".to_owned()),
                            )
                            .color(if value.unwrap_or(0) == 0 {
                                LIGHT_GREY
                            } else {
                                WHITE
                            }),
                        );
                    });
                }
            })
        });
}
