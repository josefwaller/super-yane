use std::{env, fs::File};

use iced::{Font, Settings};
use log::*;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use simplelog::{CombinedLogger, ConfigBuilder, SimpleLogger, WriteLogger};
use super_yane::Console;

mod apu_snapshot;
mod emu_state;
mod instruction_snapshot;
mod program;
#[macro_use]
mod utils;
mod engine;
mod widgets;
use program::Program;
mod disassembler;
mod table;

fn initial_state() -> Program {
    let sdl = sdl2::init().expect("Unable to init SDL");
    let audio = sdl.audio().unwrap();
    let channel: AudioQueue<f32> = audio
        .open_queue(
            None,
            &AudioSpecDesired {
                freq: Some(32_000),
                channels: Some(1),
                samples: None,
            },
        )
        .unwrap();
    info!("Channel spec is {:?}", channel.spec());
    let mut a = Program::new(channel);
    // If an environment variable was passed, load that instead
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

    iced::application(initial_state, Program::update, Program::view)
        .subscription(Program::subscription)
        .theme(Program::theme)
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
