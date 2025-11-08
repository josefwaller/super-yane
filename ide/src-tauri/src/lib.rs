extern crate super_yane;

mod webgl;

use log::*;
use serde::{Deserialize, Serialize};
use std::{ops::DerefMut, sync::Mutex, time::Duration};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ};
use tauri_plugin_log::{Target, TargetKind};
use web_sys::{
    wasm_bindgen::JsCast, window, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram,
    WebGlShader,
};

#[derive(Serialize, Deserialize, Copy, Clone)]
struct ControllerValue {
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
}

impl From<ControllerValue> for InputPort {
    fn from(value: ControllerValue) -> Self {
        InputPort::StandardController {
            a: value.a,
            b: value.b,
            x: value.x,
            y: value.y,
            up: value.up,
            left: value.left,
            right: value.right,
            down: value.down,
            start: value.start,
            select: value.select,
            r: value.r,
            l: value.l,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct UserInput {
    controllers: [ControllerValue; 2],
    reset: bool,
}

struct AppState {
    // todo make this optional
    console: Console,
    // How long the emulation has been running
    emulation_time: Duration,
}

#[tauri::command]
fn load_rom(rom_data: Vec<u8>, state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    info!("LOAD ROM");
    match state.lock() {
        Err(e) => {
            let err_msg = format!("Failed to lock state mutex: {}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
        Ok(mut guard) => {
            guard.console = Console::with_cartridge(rom_data.as_slice());
            guard.emulation_time = Duration::ZERO;
            Ok(())
        }
    }
}

#[tauri::command]
fn update_emulator(
    user_input: UserInput,
    state: tauri::State<Mutex<AppState>>,
    app: tauri::AppHandle,
) -> tauri::ipc::Response {
    match state.lock() {
        Err(e) => {
            eprintln!("Failed to lock state mutex: {}", e);
            return tauri::ipc::Response::new(vec![]);
        }
        Ok(mut guard) => {
            // Set input
            user_input
                .controllers
                .iter()
                .enumerate()
                .for_each(|(i, controller_value)| {
                    guard.console.input_ports_mut()[i] = InputPort::from(*controller_value)
                });
            // Advance a frame
            loop {
                let vblank = guard.console.ppu().is_in_vblank();
                guard.console.advance_instructions(1);
                if !vblank && guard.console.ppu().is_in_vblank() {
                    break;
                }
            }
            return tauri::ipc::Response::new(
                guard
                    .console
                    .ppu()
                    .screen_data_rgb()
                    .iter()
                    .map(|&[r, g, b]| [r, g, b, 255u8])
                    .flatten()
                    .collect::<Vec<u8>>(),
            );
        }
    }
}

#[tauri::command]
fn get_audio_samples(state: tauri::State<Mutex<AppState>>) -> tauri::ipc::Response {
    match state.lock() {
        Err(e) => {
            eprintln!("Failed to lock state mutex: {}", e);
            tauri::ipc::Response::new(vec![])
        }
        Ok(mut guard) => {
            let samples = guard
                .console
                .apu_mut()
                .sample_queue()
                .into_iter()
                .collect::<Vec<f32>>();
            let samples = samples
                .into_iter()
                .map(|s| s.to_le_bytes())
                .flatten()
                .collect::<Vec<u8>>();
            tauri::ipc::Response::new(samples)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            update_emulator,
            load_rom,
            get_audio_samples
        ])
        .manage(Mutex::new(AppState {
            console: Console::with_cartridge(
                include_bytes!("../../../app/roms/HelloWorld.sfc")
                    .to_vec()
                    .as_slice(),
            ),
            emulation_time: Duration::ZERO,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
