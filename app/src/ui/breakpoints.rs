use std::{fmt::UpperHex, sync::Arc};

use egui::{
    ComboBox, Id, Layout, Response, RichText, ScrollArea, TextEdit, Ui, Widget, WidgetWithState,
};
use strum::IntoEnumIterator;
use wdc65816::opcode_data;

use crate::{
    emulation::{
        Breakpoint::{self, Pc},
        Emulation,
    },
    ui::colors::GREY,
};

fn hex_input(ui: &mut Ui, value: &mut usize) -> Response {
    let mut str = if *value == 0 {
        "".to_owned()
    } else {
        format!("{:X}", value)
    };
    let response = ui.allocate_ui_with_layout(
        egui::Vec2::new(40.0, 15.0),
        Layout::left_to_right(egui::Align::Center),
        |ui| ui.add(TextEdit::singleline(&mut str).background_color(GREY)),
    );
    if response.inner.changed() {
        if str.trim().len() == 0 {
            *value = 0;
        } else {
            *value = usize::from_str_radix(str.trim(), 16).unwrap_or(*value);
        }
    }
    response.inner
}

#[derive(Clone)]
struct State {
    breakpoint: Breakpoint,
}
impl Default for State {
    fn default() -> Self {
        State {
            breakpoint: Breakpoint::Pc(0),
        }
    }
}

struct AdderResponse {
    response: Response,
    to_add: Option<Breakpoint>,
}

fn breakpoint_adder(ui: &mut Ui) -> AdderResponse {
    let mut to_add = None;
    let response = ui
        .horizontal(|ui| {
            // Get state
            let id = ui.unique_id();
            let mut state = ui.ctx().data_mut(|d| {
                d.get_persisted_mut_or::<State>(id, State::default())
                    .clone()
            });
            // Select type
            ComboBox::new(id, "")
                .selected_text(state.breakpoint.type_name())
                .show_ui(ui, |ui| {
                    for bp in Breakpoint::iter() {
                        if ui.selectable_label(false, bp.type_name()).clicked() {
                            state.breakpoint = bp;
                        }
                    }
                });
            match &mut state.breakpoint {
                Pc(value) => {
                    hex_input(ui, value);
                }
                _ => {}
            }
            if ui.button("Add").clicked() {
                to_add = Some(state.breakpoint);
                state = State::default();
            }
            ui.ctx().data_mut(|d| d.insert_persisted(id, state));
        })
        .response;
    AdderResponse { to_add, response }
}

pub fn breakpoints(ui: &mut Ui, emu: &mut Emulation) {
    let bp = &mut emu.breakpoints;
    ScrollArea::vertical().show_rows(ui, 15.0, bp.len(), |ui, rows| {
        for index in rows {
            ui.horizontal(|ui| {
                if let Some(bp) = bp.get(index) {
                    use Breakpoint::*;
                    let text = match bp {
                        Pc(pc) => RichText::new(format!("PC {:06X}", pc)),
                        Opcode(opcode) => {
                            let data = opcode_data(*opcode, false, false);
                            RichText::new(format!("OPCODE {:02X} ({})", opcode, data.name))
                        }
                        _ => RichText::new(""),
                    };
                    ui.label(text);
                }
            });
        }
        let AdderResponse { response, to_add } = breakpoint_adder(ui);
        if let Some(a) = to_add {
            bp.push(a);
        }
        response
    });
}
