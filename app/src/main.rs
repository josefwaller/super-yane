use std::{
    cell::RefCell,
    env,
    fs::File,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger};
use slint::{Image, Model, RenderingState, SharedPixelBuffer, SharedString, slint};

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

slint::include_modules!();

impl Into<InputPort> for StandardController {
    fn into(self) -> InputPort {
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
        } = self;
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

fn load_settings() -> Settings {
    Settings {
        is_paused: false,
        volume: 50.0,
    }
}

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
    // Load settings
    let settings = Arc::new(Mutex::new(load_settings()));
    // Initialize window
    let engine = Rc::new(RefCell::new(Engine::new(settings.clone())));
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
    // Update controllers
    let e = engine.clone();
    let ui_ptr = ui.as_weak();
    ui.on_controller_changed(move |controller| {
        // Todo: Support player 2
        // Copy controller values
        let values = [controller.into(); 2];
        e.borrow_mut().update(Command::UpdateInputPorts(values));
    });
    // Update settings
    let e = engine.clone();
    let ui_ptr = ui.as_weak();
    let settings_ptr = settings.clone();
    ui.on_settings_changed(move |s| {
        *settings_ptr.lock().unwrap() = s;
    });
    let ui_ptr = ui.as_weak();
    let e = engine.clone();
    ui.window()
        .set_rendering_notifier(move |state, _graphics| match state {
            RenderingState::AfterRendering => {
                let ui = ui_ptr.unwrap();
                e.borrow_mut().on_frame();
                let data = &e.borrow().prev_frame_data;
                let buf = SharedPixelBuffer::clone_from_slice(data.as_flattened(), 256, 224);
                ui.set_pixel_data(Image::from_rgb8(buf));
            }
            _ => {}
        })
        .unwrap();
    ui.run().expect("Unable to start Slint application");
}
