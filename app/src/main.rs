use closure::closure;
use std::{
    cell::RefCell,
    env,
    fs::File,
    rc::Rc,
    sync::{Arc, Mutex},
};

use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger};
use slint::{Image, ModelRc, RenderingState, SharedPixelBuffer, VecModel};

mod apu_snapshot;
mod audio;
mod cpu_snapshot;
mod utils;
// mod emu_state;
// mod program;
// #[macro_use]
// mod utils;
mod engine;
// mod widgets;
// use program::Program;

use crate::engine::{AdvanceAmount, Command, Engine};
use super_yane::{Console, InputPort};
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
    let ui_ptr = ui.as_weak();
    // Load settings
    let settings = Arc::new(Mutex::new(load_settings()));
    // Create console data struct
    let data = Arc::new(Mutex::new(ConsoleData::default()));
    // Initialize window
    let engine = Rc::new(RefCell::new(Engine::new(settings.clone(), data.clone())));
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
    ui.on_controller_changed(closure!(clone engine, |controller| {
        // Todo: Support player 2
        // Copy controller values
        let values = [controller.into(); 2];
        engine.borrow_mut().update(Command::UpdateInputPorts(values));
    }));
    // Update settings
    ui.on_settings_changed(closure!(clone settings, |s| {
        *settings.lock().unwrap() = s;
    }));
    // Advance instructions
    ui.on_advance_instructions(closure!(clone engine, |n| {
        engine.borrow_mut().update(Command::Advance(AdvanceAmount::Instructions(n as u32)));
    }));
    // Advance frames
    ui.on_advance_frames(closure!(clone engine, |n| {
        engine.borrow_mut().update(Command::Advance(AdvanceAmount::Frames(n as u32)));
    }));
    ui.window()
        .set_rendering_notifier(
            closure!(clone ui_ptr, clone engine, |state, _graphics| match state {
                RenderingState::AfterRendering => {
                    let ui = ui_ptr.unwrap();
                    engine.borrow_mut().on_frame();
                    let e = engine.borrow();
                    let buf = SharedPixelBuffer::clone_from_slice(e.prev_frame_data.as_flattened(), 256, 224);
                    ui.set_pixel_data(Image::from_rgb8(buf));
                    ui.set_console_data(e.console_data());
                    {
                    let c = e.console();
                    let pc = c.pc();
                    ui.set_disassembly_lines(ModelRc::new(VecModel::from(e.disassembly_lines(
                        c.cartridge().transform_address(pc)
                    ))));
                }
                }
                _ => {}
            }),
        )
        .unwrap();
    ui.run().expect("Unable to start Slint application");
}
