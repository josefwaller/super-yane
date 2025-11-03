extern crate super_yane;

mod webgl;

use log::*;
use std::{ops::DerefMut, sync::Mutex, time::Duration};
use super_yane::{Console, MASTER_CLOCK_SPEED_HZ};
use tauri_plugin_log::{Target, TargetKind};
use web_sys::{
    wasm_bindgen::JsCast, window, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram,
    WebGlShader,
};

struct AppState {
    // todo make this optional
    console: Console,
    // How long the emulation has been running
    emulation_time: Duration,
}
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
// fn update_emulator(duration_millis: u64) {
fn update_emulator(
    duration_millis: u64,
    state: tauri::State<Mutex<AppState>>,
    app: tauri::AppHandle,
) -> tauri::ipc::Response {
    match state.lock() {
        Err(e) => {
            eprintln!("Failed to lock state mutex: {}", e);
            return tauri::ipc::Response::new(vec![]);
        }
        Ok(mut guard) => {
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
        .invoke_handler(tauri::generate_handler![update_emulator, greet, load_rom])
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
