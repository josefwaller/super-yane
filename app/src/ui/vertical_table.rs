use egui::{Color32, Label, Margin, RichText, Ui};
use egui_extras::{Column, TableBuilder};

#[derive(Clone)]
pub struct Row<'a> {
    name: &'a str,
    value: String,
    color: Option<Color32>,
    indent: u32,
}
impl<'a> Row<'a> {
    pub fn new(name: &'a str, value: String) -> Row<'a> {
        Row {
            name,
            value,
            color: None,
            indent: 0,
        }
    }
    pub fn color(mut self, color: Color32) -> Row<'a> {
        self.color = Some(color);
        self
    }
    pub fn indent(mut self, indent: u32) -> Row<'a> {
        self.indent = indent;
        self
    }
}

pub fn vertical_table(ui: &mut Ui, row_data: &[Row], header_color: Color32) {
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .body(|body| {
            body.rows(16.0, row_data.len(), |mut row| {
                let index = row.index();
                let data = row_data[index].clone();
                row.col(|ui| {
                    let label = Label::new(
                        RichText::new(data.name.to_owned())
                            .color(data.color.unwrap_or(header_color)),
                    )
                    .wrap_mode(egui::TextWrapMode::Extend);
                    egui::Frame::NONE
                        .inner_margin(Margin {
                            left: 20 * data.indent as i8,
                            ..Margin::ZERO
                        })
                        .show(ui, |ui| {
                            ui.add(label);
                        });
                });
                row.col(|ui| {
                    let label = Label::new(RichText::new(data.value).color(Color32::WHITE))
                        .wrap_mode(egui::TextWrapMode::Extend);
                    ui.add(label);
                });
            });
        });
}
