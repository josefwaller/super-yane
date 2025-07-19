#[derive(Copy, Clone, Default, Debug)]
pub struct StandardControllerValue {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub start: bool,
    pub select: bool,
    pub r: bool,
    pub l: bool,
}

#[derive(Copy, Clone)]
pub enum InputPort {
    Empty,
    StandardController(StandardControllerValue),
}
