use std::{env, fs::File, time::Duration};

use iced::{Font, Settings};
use log::*;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger};
use slint::{Image, SharedPixelBuffer, slint};

mod apu_snapshot;
mod cpu_snapshot;
// mod emu_state;
// mod program;
// #[macro_use]
// mod utils;
mod engine;
// mod widgets;
// use program::Program;

use crate::engine::{AdvanceSettings, Engine};
mod disassembler;
mod profiler;
mod table;

slint::include_modules!();

// fn initial_state() -> Program {
//     let sdl = sdl2::init().expect("Unable to init SDL");
//     let audio = sdl.audio().unwrap();
//     let channel: AudioQueue<f32> = audio
//         .open_queue(
//             None,
//             &AudioSpecDesired {
//                 freq: Some(32_000),
//                 channels: Some(1),
//                 samples: None,
//             },
//         )
//         .unwrap();
//     info!("Channel spec is {:?}", channel.spec());
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

    let ui = AppWindow::new().unwrap();
    let mut engine = Engine::new();
    let ui_ptr = ui.as_weak();
    ui.on_advance_emulator(move || {
        engine.advance_dt(Duration::from_millis(16), AdvanceSettings::default());
        engine.on_frame();
        let data = &engine.prev_frame_data;
        let buf = SharedPixelBuffer::clone_from_slice(data.as_flattened(), 256, 240);
        ui_ptr.unwrap().set_pixel_data(Image::from_rgb8(buf));
    });
    ui.run().expect("Unable to start Slint application");

    // iced::application(initial_state, Program::update, Program::view)
    //     .subscription(Program::subscription)
    //     .theme(Program::theme)
    //     .settings(Settings {
    //         id: None,
    //         vsync: false,
    //         fonts: vec![],
    //         default_font: Font::MONOSPACE,
    //         default_text_size: 12.into(),
    //         antialiasing: false,
    //     })
    //     .exit_on_close_request(false)
    //     .run()
    //     .unwrap();
}
