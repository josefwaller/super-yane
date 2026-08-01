use egui::Key;

/// The keybingins of Egui key -> SNES controller button.
/// Right now only contains standard controller buttons.
pub struct Keybindings {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
    pub a: Key,
    pub b: Key,
    pub x: Key,
    pub y: Key,
    pub l: Key,
    pub r: Key,
    pub start: Key,
    pub select: Key,
}

pub fn get_initial_keybindings() -> Keybindings {
    Keybindings {
        up: Key::W,
        down: Key::S,
        left: Key::A,
        right: Key::D,
        a: Key::B,
        b: Key::Space,
        x: Key::N,
        y: Key::M,
        l: Key::Q,
        r: Key::E,
        start: Key::R,
        select: Key::F,
    }
}
