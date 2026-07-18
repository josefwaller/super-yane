use std::{fmt::UpperHex, sync::Arc};

use egui::{
    ComboBox, Id, Layout, Response, RichText, ScrollArea, TextEdit, Ui, Widget, WidgetWithState,
};
use egui_extras::{Column, TableBuilder};
use strum::IntoEnumIterator;
use wdc65816::{format_address_mode, opcode_data};

use crate::{
    emulation::{Breakpoint, Emulation},
    ui::colors::{GREEN_PRIMARY, GREY, LIGHT_BLUE_PRIMARY, RED_PRIMARY},
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
            use Breakpoint::*;
            match &mut state.breakpoint {
                Pc(value) => {
                    hex_input(ui, value);
                }
                Opcode(opcode) => {
                    let mut value = *opcode as usize;
                    hex_input(ui, &mut value);
                    *opcode = value.clamp(u8::MIN as usize, u8::MAX as usize) as u8;
                }
                Dma(index) => {
                    hex_input(ui, index);
                }
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
    // Row index to remove
    let mut to_remove: Option<usize> = None;
    TableBuilder::new(ui)
        .columns(Column::remainder(), 2)
        .column(Column::auto())
        .header(15.0, |mut row| {
            row.col(|ui| {
                ui.label("TYPE");
            });
            row.col(|ui| {
                ui.label("ARGS");
            });
            row.col(|_| {});
        })
        .body(|body| {
            body.rows(15.0, bp.len(), |mut row| {
                let index = row.index();
                if let Some(bp) = bp.get(index) {
                    use Breakpoint::*;
                    let (left, right) = match bp {
                        Pc(pc) => (
                            RichText::new("PC").color(RED_PRIMARY),
                            format!("{:06X}", pc),
                        ),
                        Opcode(opcode) => {
                            let data = opcode_data(*opcode, false, false);
                            (
                                RichText::new("OPCODE").color(GREEN_PRIMARY),
                                format!("{:02X} ({})", opcode, data.name),
                            )
                        }
                        Dma(index) => (
                            RichText::new("DMA").color(LIGHT_BLUE_PRIMARY),
                            format!("{:X}", index),
                        ),
                    };
                    row.col(|ui| {
                        ui.label(left);
                    });
                    row.col(|ui| {
                        ui.label(right);
                    });
                    row.col(|ui| {
                        if ui.button("X").clicked() {
                            to_remove = Some(index);
                        }
                    });
                }
            });
        });
    let AdderResponse { response, to_add } = breakpoint_adder(ui);
    if let Some(r) = to_remove {
        bp.remove(r);
    }
    if let Some(a) = to_add {
        bp.push(a);
    }
}
