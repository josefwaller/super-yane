use egui::{Event::Key, Id};
use egui_extras::{Column, TableBuilder};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use crate::{emulation::Emulation, keybindings::Keybindings, ui::settings::EmuButton::Up};

/// All the potential buttons that need a keybinding
#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Copy, Debug)]
pub enum EmuButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Start,
    Select,
    L,
    R,
}

#[derive(Clone)]
struct State {
    set_keybinding: Option<EmuButton>,
}

fn get_key<'a>(emu: &'a mut Emulation, button: EmuButton) -> &'a mut egui::Key {
    let kb = &mut emu.keybindings;
    use EmuButton::*;
    match button {
        Up => &mut kb.up,
        Down => &mut kb.down,
        Left => &mut kb.left,
        Right => &mut kb.right,
        A => &mut kb.a,
        B => &mut kb.b,
        X => &mut kb.x,
        Y => &mut kb.y,
        Start => &mut kb.start,
        Select => &mut kb.select,
        L => &mut kb.l,
        R => &mut kb.r,
    }
}

pub fn settings(ui: &mut egui::Ui, emu: &mut Emulation) {
    // All settings have the same state
    let id = Id::from("SETTINGS");
    let mut state = ui.ctx().data_mut(|data| {
        data.get_persisted(id).unwrap_or(State {
            set_keybinding: None,
        })
    });
    TableBuilder::new(ui)
        .columns(Column::auto(), 2)
        .header(18.0, |mut row| {
            row.col(|c| {
                c.label("Button");
            });
            row.col(|c| {
                c.label("Binding");
            });
        })
        .body(|mut body| {
            for binding in EmuButton::iter() {
                let b = get_key(emu, binding);
                body.row(1.8, |mut row| {
                    row.col(|c| {
                        c.label(binding.to_string());
                    });
                    row.col(|c| {
                        if c.button(if state.set_keybinding == Some(binding) {
                            "[PRESS NEW KEY]".to_string()
                        } else {
                            format!("{:?}", b)
                        })
                        .clicked()
                        {
                            state.set_keybinding = Some(binding);
                        }
                    });
                });
            }
        });
    if let Some(key) = state.set_keybinding {
        ui.ctx().input(|i| {
            for e in i.events.iter() {
                match e {
                    Key { key: k, .. } => {
                        *get_key(emu, key) = k.clone();
                        state.set_keybinding = None;
                    }
                    _ => {}
                }
            }
        })
    }
    ui.data_mut(|data| data.insert_persisted(id, state));
}
