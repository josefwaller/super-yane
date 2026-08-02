use gilrs::GamepadId;

#[derive(Copy, Clone, Debug)]
pub enum Input {
    Key(egui::Key),
    Gamepad(gilrs::GamepadId, gilrs::Button),
}

#[derive(PartialEq)]
pub enum InputSource {
    Keyboard,
    Gamepad(GamepadId),
}

/// The keybingins of Egui key -> SNES controller button.
/// Right now only contains standard controller buttons.
pub struct Keybindings {
    /// The source of this keybindings, either the keyboard or a specific gamepad
    pub source: InputSource,
    pub up: Input,
    pub down: Input,
    pub left: Input,
    pub right: Input,
    pub a: Input,
    pub b: Input,
    pub x: Input,
    pub y: Input,
    pub l: Input,
    pub r: Input,
    pub start: Input,
    pub select: Input,
}

pub fn get_default_keyboard_keybindings() -> Keybindings {
    Keybindings {
        source: InputSource::Keyboard,
        up: Input::Key(egui::Key::W),
        left: Input::Key(egui::Key::A),
        right: Input::Key(egui::Key::D),
        down: Input::Key(egui::Key::S),
        a: Input::Key(egui::Key::B),
        b: Input::Key(egui::Key::Space),
        x: Input::Key(egui::Key::N),
        y: Input::Key(egui::Key::M),
        start: Input::Key(egui::Key::R),
        select: Input::Key(egui::Key::F),
        l: Input::Key(egui::Key::Q),
        r: Input::Key(egui::Key::E),
    }
}

pub fn get_default_gamepad_bindings(id: GamepadId) -> Keybindings {
    Keybindings {
        source: InputSource::Gamepad(id),
        up: Input::Gamepad(id, gilrs::Button::DPadUp),
        left: Input::Gamepad(id, gilrs::Button::DPadLeft),
        right: Input::Gamepad(id, gilrs::Button::DPadRight),
        down: Input::Gamepad(id, gilrs::Button::DPadDown),
        a: Input::Gamepad(id, gilrs::Button::East),
        b: Input::Gamepad(id, gilrs::Button::South),
        x: Input::Gamepad(id, gilrs::Button::North),
        y: Input::Gamepad(id, gilrs::Button::West),
        start: Input::Gamepad(id, gilrs::Button::Start),
        select: Input::Gamepad(id, gilrs::Button::Select),
        l: Input::Gamepad(id, gilrs::Button::LeftTrigger),
        r: Input::Gamepad(id, gilrs::Button::RightTrigger),
    }
}
