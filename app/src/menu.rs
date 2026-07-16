//! Handles the menu bar initialization and handling events
use std::{
    error::Error,
    str::FromStr,
    sync::{Arc, Mutex},
};

use egui::Id;
use egui_dock::DockState;
use muda::{Menu, MenuEvent, MenuItem, Submenu};
use rfd::FileDialog;
use strum::{Display, EnumString};
use super_yane::Console;

use crate::{emulation::Emulation, ui::EmuTab};

#[derive(EnumString, Display)]
pub enum MenuCommand {
    Quit,
    LoadRom,
    LoadSram,
    LoadSavestate,
    SaveSram,
    SaveSavestate,
}

pub fn initialize_menu() -> Result<Menu, Box<dyn Error>> {
    use MenuCommand::*;
    // Initialize menu bar
    let menu = Menu::with_items(&[
        &Submenu::with_items("App", true, &[&MenuItem::with_id(Quit, "Quit", true, None)])?,
        &Submenu::with_items(
            "File",
            true,
            &[
                &Submenu::with_items(
                    "Load",
                    true,
                    &[
                        &MenuItem::with_id(LoadRom, "ROM", true, None),
                        &MenuItem::with_id(LoadSram, "SRAM", true, None),
                        &MenuItem::with_id(LoadSavestate, "Savestate", true, None),
                    ],
                )?,
                &Submenu::with_items(
                    "Save",
                    true,
                    &[
                        &MenuItem::with_id(SaveSram, "SRAM", true, None),
                        &MenuItem::with_id(SaveSavestate, "Savestate", true, None),
                    ],
                )?,
            ],
        )?,
    ])?;

    #[cfg(target_os = "windows")]
    unsafe {
        // todo get windows handle in windows
        menu.init_for_hwnd()
    };
    #[cfg(target_os = "linux")]
    menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();
    Ok(menu)
}

fn read_file(filter_name: &str, filters: &[&str]) -> Option<Vec<u8>> {
    FileDialog::new()
        .add_filter(filter_name, filters)
        .pick_file()
        .map(|f| {
            std::fs::read(&f)
                .map_err(|e| log::error!("Unable to read file {:?}: {:?}", f, e))
                .ok()
        })
        .flatten()
}

fn write_file(filter: &str, ext: &[&str], bytes: &[u8]) {
    FileDialog::new()
        .add_filter(filter, ext)
        .save_file()
        .map(|f| {
            std::fs::write(&f, bytes)
                .map_err(|e| log::error!("Unable to write to file {:?}: {:?}", f, e))
        });
}

pub fn spawn_menu_thread(emu: Arc<Mutex<Emulation>>, tree: Arc<Mutex<DockState<(Id, EmuTab)>>>) {
    // Get menu item
    std::thread::Builder::new()
        .name("Super Y.A.N.E. MenuBar Handler".to_owned())
        .stack_size(8_388_608)
        .spawn(move || {
            loop {
                let m = MenuEvent::receiver().recv();
                match m {
                    Ok(event) => match MenuCommand::from_str(event.id().0.as_str()) {
                        Ok(c) => {
                            use MenuCommand::*;
                            match c {
                                LoadRom => {
                                    read_file("Super Nintendo ROMs", &["sfc", "smc"]).map(
                                        |bytes| {
                                            emu.lock().unwrap().console =
                                                Console::with_cartridge(&bytes)
                                        },
                                    );
                                }
                                LoadSram => {
                                    read_file("SRAM data", &["bin"]).map(|bytes| {
                                        emu.lock().unwrap().load_sram(&bytes);
                                    });
                                }
                                LoadSavestate => {
                                    read_file("Super Y.A.N.E Savestate", &["bin"]).map(|bytes| {
                                        match serde_brief::from_slice::<Console>(&bytes) {
                                            Ok(c) => {
                                                emu.lock().unwrap().load_savestate(c);
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Unable to deserialize savestate: {:?}",
                                                    e
                                                )
                                            }
                                        }
                                    });
                                }
                                SaveSram => {
                                    let data = emu.lock().unwrap().console.cartridge().sram.clone();
                                    write_file("SRAM data", &["bin"], &data);
                                }
                                SaveSavestate => {
                                    let data = {
                                        serde_brief::to_vec(&emu.lock().unwrap().console).unwrap()
                                    };
                                    write_file("Super Y.A.N.E. Savestate", &["bin"], &data);
                                }
                                Quit => emu
                                    .lock()
                                    .unwrap()
                                    .ui_ctx
                                    .send_viewport_cmd(egui::ViewportCommand::Close),
                            }
                        }
                        Err(e) => log::error!("Invalid menu id received: {}", e),
                    },
                    Err(_) => {}
                }
            }
        })
        .unwrap();
}
