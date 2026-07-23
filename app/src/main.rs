mod app;
pub mod apu_snapshot;
pub mod audio;
pub mod cpu_snapshot;
pub mod disassembler;
pub mod emulation;
pub mod engine;
mod menu;
pub mod profiler;
pub mod ui;
pub mod utils;

use app::App;
use clap::Parser;
use egui::{FontData, FontDefinitions, FontFamily, FontId};
use log::{debug, error};
use objc2::MainThreadMarker;
use objc2_app_kit::NSWindow;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger};
use std::{
    env,
    error::Error,
    fmt::format,
    fs::File,
    io::BufWriter,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use super_yane::Console;

use crate::{
    emulation::Emulation,
    engine::{Command, Engine},
    menu::initialize_menu,
};

#[derive(Debug)]
enum LoadConsoleError {
    FileError(std::io::Error),
    DeserializationError(serde_brief::Error),
}

#[derive(Parser, Debug)]
struct Args {
    file: Option<PathBuf>,
    #[arg(short, long, default_value_t = false)]
    paused: bool,
    #[arg(short, long, default_value_t = 5.0)]
    volume: f32,
}

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");

fn initial_console(arg: Option<PathBuf>) -> Result<Console, LoadConsoleError> {
    // Load ROM/savestate
    match arg {
        Some(f) => match std::fs::read(&f) {
            Ok(bytes) => {
                if f.ends_with(".bin") {
                    let mut c: Console = serde_brief::from_slice(&bytes)
                        .map_err(LoadConsoleError::DeserializationError)?;
                    c.ppu_mut().reset_vram_cache();
                    Ok(c)
                } else {
                    Ok(Console::with_cartridge(&bytes))
                }
            }
            Err(e) => {
                error!("Unable to read file {:?}: {:?}", f, e);
                Err(LoadConsoleError::FileError(e))
            }
        },
        None => Ok(Console::with_cartridge(DEFAULT_CARTRIDGE)),
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // Initialize logger
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

    // Initialize window
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1200.0])
            // .with_decorations(false)
            .with_taskbar(false)
            .with_title("Super Y.A.N.E"),
        ..Default::default()
    };
    // Don't allow automatic window tabbing to remove View menu bar options
    NSWindow::setAllowsAutomaticWindowTabbing(false, unsafe { MainThreadMarker::new_unchecked() });

    // Load font
    macro_rules! FONT_PATH {
        () => {
            "../assets/VeraMono.ttf"
        };
    }
    let font_data = include_bytes!(FONT_PATH!());
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        FONT_PATH!().to_owned(),
        Arc::new(FontData::from_static(font_data)),
    );
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, FONT_PATH!().to_owned());
    // Run
    eframe::run_native(
        "Super Y.A.N.E",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(fonts);
            cc.egui_ctx.global_style_mut(|s| {
                s.text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(12.0, FontFamily::Monospace),
                );
            });
            #[cfg(debug_assertions)]
            cc.egui_ctx
                .global_style_mut(|s| s.debug.warn_if_rect_changes_id = false);
            // Initialize emulation
            let console = initial_console(args.file).unwrap();
            // TODO: Add settings to Emulation to parse from args
            let emu = Arc::new(Mutex::new(Emulation::new(console, cc.egui_ctx.clone())));
            emu.lock().unwrap().is_paused = args.paused;
            emu.lock().unwrap().volume = args.volume;
            Ok(Box::new(App::new(cc, emu, initialize_menu().unwrap())))
        }),
    )?;
    Ok(())
}
