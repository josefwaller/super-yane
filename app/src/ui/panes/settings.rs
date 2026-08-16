use egui::{ComboBox, Event::Key, Id};
use egui_extras::{Column, TableBuilder};
use gilrs::{Event, EventType, Gilrs};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use crate::{
    emulation::Emulation,
    keybindings::{
        self, Input,
        InputSource::{self, Gamepad, Keyboard},
        Keybindings, get_default_gamepad_bindings, get_default_keyboard_keybindings,
    },
};

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
    editing_keybinding: Option<EmuButton>,
}

fn get_key<'a>(kb: &'a mut Keybindings, button: EmuButton) -> &'a mut Input {
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

pub fn settings(ui: &mut egui::Ui, emu: &mut Emulation, gilrs: &mut Gilrs) {
    // All settings have the same state
    let id = Id::from("SETTINGS");
    let mut state = ui.ctx().data_mut(|data| {
        data.get_persisted(id).unwrap_or(State {
            editing_keybinding: None,
        })
    });
    ComboBox::from_label("Source")
        .selected_text(match emu.keybindings.source {
            InputSource::Keyboard => "Keyboard".to_owned(),
            InputSource::Gamepad(id) => gilrs.gamepad(id).name().to_owned(),
        })
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(emu.keybindings.source == InputSource::Keyboard, "Keyboard")
                .clicked()
            {
                emu.keybindings = get_default_keyboard_keybindings();
            }
            for (id, gp) in gilrs.gamepads() {
                if ui
                    .selectable_label(
                        emu.keybindings.source == InputSource::Gamepad(id),
                        gp.name(),
                    )
                    .clicked()
                {
                    emu.keybindings = get_default_gamepad_bindings(id);
                }
            }
            while let Some(_) = gilrs.next_event() {}
            gilrs.inc();
        });
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
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
                let b = get_key(&mut emu.keybindings, binding);
                body.row(1.8, |mut row| {
                    row.col(|c| {
                        c.label(binding.to_string());
                    });
                    row.col(|c| {
                        if c.button(if state.editing_keybinding == Some(binding) {
                            "[PRESS NEW KEY]".to_string()
                        } else {
                            format!("{:?}", b)
                        })
                        .clicked()
                        {
                            state.editing_keybinding = Some(binding);
                        }
                    });
                });
            }
        });
    if let Some(key) = state.editing_keybinding {
        // Check for keyboard input
        match emu.keybindings.source {
            InputSource::Keyboard => {
                ui.ctx().input(|i| {
                    for e in i.events.iter() {
                        match e {
                            Key { key: k, .. } => {
                                *get_key(&mut emu.keybindings, key) = Input::Key(k.clone());
                                state.editing_keybinding = None;
                            }
                            _ => {}
                        }
                    }
                });
            }
            InputSource::Gamepad(id) => {
                // Check for controller input
                while let Some(ev) = gilrs.next_event() {
                    match ev.event {
                        EventType::ButtonPressed(button, _) => {
                            if ev.id == id {
                                *get_key(&mut emu.keybindings, key) = Input::Gamepad(id, button);
                                state.editing_keybinding = None;
                            }
                        }
                        _ => {
                            // Noop
                        }
                    }
                }
            }
        }
    }
    ui.data_mut(|data| data.insert_persisted(id, state));
}
