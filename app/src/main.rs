use std::{
    env,
    fs::File,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use iced::{Font, Settings};
use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, SimpleLogger, WriteLogger};
use super_yane::{Console, MASTER_CLOCK_SPEED_HZ};

mod application;
mod apu_snapshot;
mod emu_state;
mod instruction_snapshot;
#[macro_use]
mod utils;
mod engine;
mod widgets;
use application::Application;
use emu_state::EmuState;
mod table;

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");

fn initial_state() -> Application {
    let mut a = Application::default();
    match env::args().nth(1) {
        Some(f) => match std::fs::read(&f) {
            Ok(bytes) => {
                debug!("Reading {}", f);
                if f.ends_with(".sy.bin") {
                    a.engine.load_savestate(&bytes);
                } else {
                    a.engine.load_rom(&bytes)
                }
            }
            Err(e) => {
                error!("Unable to read file {}: {:?}", f, e);
            }
        },
        None => {}
    };
    a
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
        SimpleLogger::new(log::LevelFilter::Debug, config),
    ])
    .unwrap();
    info!("Logger initialized");

    iced::application(initial_state, Application::update, Application::view)
        .subscription(Application::subscription)
        .theme(Application::theme)
        .settings(Settings {
            id: None,
            vsync: false,
            fonts: vec![],
            default_font: Font::MONOSPACE,
            default_text_size: 12.into(),
            antialiasing: false,
        })
        .exit_on_close_request(false)
        .run()
        .unwrap();
}
