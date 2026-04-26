use std::{cell::RefCell, env, fs::File, rc::Rc, time::Instant};

use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger};
use slint::{Image, RenderingState, SharedPixelBuffer, slint};

mod apu_snapshot;
mod audio;
mod cpu_snapshot;
// mod emu_state;
// mod program;
// #[macro_use]
// mod utils;
mod engine;
// mod widgets;
// use program::Program;

use crate::engine::{AdvanceSettings, Command, Engine};
use super_yane::InputPort;
mod disassembler;
mod profiler;
mod table;

impl From<StandardController> for InputPort {
    fn from(value: StandardController) -> Self {
        let StandardController {
            a,
            b,
            x,
            y,
            up,
            left,
            right,
            down,
            start,
            select,
            r,
            l,
        } = value;
        InputPort::StandardController {
            a,
            b,
            x,
            y,
            up,
            left,
            right,
            down,
            start,
            select,
            r,
            l,
        }
    }
}

slint::include_modules!();

// fn initial_state() -> Program {

//     let mut a = Program::new(channel);
//     // If an environment variable was passed, load that instead
//     match env::args().nth(1) {
//         Some(f) => match std::fs::read(&f) {
//             Ok(bytes) => {
//                 debug!("Reading {}", f);
//                 if f.ends_with(".sy.bin") {
//                     a.engine.load_savestate(&bytes);
//                 } else {
//                     a.engine.load_rom(&bytes)
//                 }
//             }
//             Err(e) => {
//                 error!("Unable to read file {}: {:?}", f, e);
//             }
//         },
//         None => {}
//     };
//     a
// }
fn main() {
    let config = ConfigBuilder::new()
        .add_filter_allow_str("app")
        .add_filter_allow_str("super_yane")
        .add_filter_allow_str("spc700")
        .add_filter_allow_str("wdc65816")
        .build();
    CombinedLogger::init(vec![
        WriteLogger::new(
            log::LevelFilter::Debug,
            config.clone(),
            File::create("./super_yane.log").unwrap(),
        ),
        TermLogger::new(
            log::LevelFilter::Debug,
            config,
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Always,
        ),
    ])
    .unwrap();
    info!("Logger initialized");
    // Initialize UI
    let ui = AppWindow::new().unwrap();
    // Initialize window
    let engine = Rc::new(RefCell::new(Engine::new()));
    // Load ROM/savestate
    match env::args().nth(1) {
        Some(f) => match std::fs::read(&f) {
            Ok(bytes) => {
                debug!("Reading {}", f);
                if f.ends_with(".sy.bin") {
                    engine.borrow_mut().load_savestate(&bytes);
                } else {
                    engine.borrow_mut().load_rom(&bytes)
                }
            }
            Err(e) => {
                error!("Unable to read file {}: {:?}", f, e);
            }
        },
        None => {}
    };
    let ui_ptr = ui.as_weak();
    let e = engine.clone();
    ui.on_controller_changed(move || {
        e.borrow_mut().update(Command::UpdateInputPorts(
            [InputPort::from(ui_ptr.unwrap().get_controller()); 2],
        ))
    });
    let ui_ptr = ui.as_weak();
    let e = engine.clone();
    ui.window()
        .set_rendering_notifier(move |state, _graphics| match state {
            RenderingState::AfterRendering => {
                let ui = ui_ptr.unwrap();
                engine.borrow_mut().on_frame();
                let data = &e.borrow().prev_frame_data;
                let buf = SharedPixelBuffer::clone_from_slice(data.as_flattened(), 256, 224);
                ui.set_pixel_data(Image::from_rgb8(buf));
            }
            _ => {}
        })
        .unwrap();
    ui.run().expect("Unable to start Slint application");
}
