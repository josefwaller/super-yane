use closure::closure;
use rfd::FileDialog;
use std::{
    cell::RefCell,
    env,
    fs::{File, write},
    rc::Rc,
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
use super_yane::{Cpu, InputPort};
mod disassembler;
mod profiler;
mod table;

slint::include_modules!();

impl Into<InputPort> for StandardController {
    fn into(self) -> InputPort {
        // let mut port = InputPort::default_standard_controller();
        // copy_fields!(
        //     self, port, a, b, x, y, up, left, right, down, start, select, r, l
        // );
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
    // Update controllers
    ui.on_controller_changed(closure!(clone engine, |controller| {
        // Todo: Support player 2
        // Copy controller values
        let values = [controller.into(); 2];
        engine.borrow_mut().update(Command::UpdateInputPorts(values));
    }));
    // Update settings
    ui.on_settings_changed(closure!(clone engine, |s| {
        engine.borrow_mut().update_settings(s);
    }));
    // Advance instructions
    ui.on_advance_instructions(closure!(clone engine, |n| {
        engine.borrow_mut().update(Command::Advance(AdvanceAmount::Instructions(n as u32)));
    }));
    // Advance frames
    ui.on_advance_frames(closure!(clone engine, |n| {
        engine.borrow_mut().update(Command::Advance(AdvanceAmount::Frames(n as u32)));
    }));
    ui.on_reset(closure!(clone engine, || {
        engine.borrow_mut().update(Command::Reset);
    }));
    // Define rust functions
    let funcs = ui.global::<ExternalFunction>();
    funcs.on_byte_to_hex(|b| format!("{:02X}", b).into());
    funcs.on_word_to_hex(|b| format!("{:04X}", b).into());
    funcs.on_addr_to_hex(|b| format!("{:06X}", b).into());
    ui.on_load_rom(closure!(clone engine, || {
        match FileDialog::new().add_filter("Super NES Rom", &["rom", "sfc"]).pick_file() {
            None => {}
            Some(path) => {
                match std::fs::read(&path) {
                    Err(e) => {
                        error!("Unable to read file {:?}: {:?}", &path, e);
                    }
                    Ok(bytes) => {
                        engine.borrow_mut().update(Command::LoadRom(bytes))
                    }
                }
            }
        }
    }));
    ui.on_save_savestate(closure!(clone engine, || {
        match FileDialog::new()
            .add_filter("Super Y.A.N.E. Savestate", &["sy.bin"])
            .set_title("Save game state")
            .save_file() {
            None => {},
            Some(path) => {
                let data = engine.borrow().get_savestate();
                match std::fs::write(&path, &data) {
                    Ok(_) => {},
                    Err(e) => error!("Unable to write to file {:?}: {:?}", path, e)
                }
            }
        }
    }));
    ui.on_load_savestate(closure!(clone engine, || {
        match FileDialog::new()
            .add_filter("Super Y.A.N.E. Savestate", &["sy.bin"])
            .set_title("Load game state")
            .pick_file() {
                None => {},
                Some(path) => {
                    match std::fs::read(&path) {
                        Ok(bytes) => engine.borrow_mut().load_savestate(&bytes).unwrap(),
                        Err(e) => error!("Unable to read file {:?}: {:?}", &path, e)
                    }
                }
            }
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
                        ui.set_backgrounds(ModelRc::from(
                            Rc::from(VecModel::from_iter(
                                c.ppu().backgrounds.iter().map(|b| b.into())
                            ))
                        ));
                    }
                    // Set up RAM information
                    e.refresh_binary_data(ui);
                }
                _ => {}
            }),
        )
        .unwrap();
    ui.run().expect("Unable to start Slint application");
}
