#[derive(Copy, Clone, Default, Debug)]
pub enum InputPort {
    #[default]
    Empty,
    StandardController {
        a: bool,
        b: bool,
        x: bool,
        y: bool,
        up: bool,
        left: bool,
        right: bool,
        down: bool,
        start: bool,
        select: bool,
        r: bool,
        l: bool,
    },
}

impl InputPort {
    pub fn default_standard_controller() -> InputPort {
        InputPort::StandardController {
            a: false,
            b: false,
            x: false,
            y: false,
            up: false,
            left: false,
            right: false,
            down: false,
            start: false,
            select: false,
            r: false,
            l: false,
        }
    }
}
