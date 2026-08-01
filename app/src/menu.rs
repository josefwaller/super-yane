//! Handles the menu bar initialization and handling events
use std::{
    error::Error,
    str::FromStr,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use egui::Id;
use egui_dock::DockState;
use muda::{Menu, MenuEvent, MenuItem, Submenu};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use super_yane::Console;

use crate::{emulation::Emulation, ui::EmuTab};

#[derive(Serialize, Deserialize, Debug)]
pub enum SourceType {
    Rom,
    Sram,
    Savestate,
}

#[derive(Serialize, Deserialize)]
pub enum MenuCommand {
    OpenSettings,
    Quit,
    Load(SourceType),
    Save(SourceType),
    OpenTab(EmuTab),
}

impl ToString for MenuCommand {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl FromStr for MenuCommand {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn initialize_menu() -> Result<Menu, Box<dyn Error>> {
    use MenuCommand::*;
    use SourceType::*;
    // Initialize menu bar
    let menu = Menu::with_items(&[
        &Submenu::with_items(
            "App",
            true,
            &[
                &MenuItem::with_id(OpenSettings, "Settings", true, None),
                &MenuItem::with_id(Quit, "Quit", true, None),
            ],
        )?,
        &Submenu::with_items(
            "File",
            true,
            &[
                &Submenu::with_items(
                    "Load",
                    true,
                    &[
                        &MenuItem::with_id(Load(Rom), "ROM", true, None),
                        &MenuItem::with_id(Load(Sram), "SRAM", true, None),
                        &MenuItem::with_id(Load(Savestate), "Savestate", true, None),
                    ],
                )?,
                &Submenu::with_items(
                    "Save",
                    true,
                    &[
                        &MenuItem::with_id(Save(Sram), "SRAM", true, None),
                        &MenuItem::with_id(Save(Savestate), "Savestate", true, None),
                    ],
                )?,
            ],
        )?,
        &Submenu::with_items(
            "View",
            true,
            &[&{
                let s = Submenu::new("Tab", true);
                for tab in EmuTab::iter() {
                    s.append(&MenuItem::with_id(
                        OpenTab(tab),
                        tab.to_string(),
                        true,
                        None,
                    ))?
                }
                s
            }],
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
                            use SourceType::*;
                            match c {
                                Load(src) => match src {
                                    Rom => {
                                        read_file("Super Nintendo ROMs", &["sfc", "smc"]).map(
                                            |bytes| {
                                                emu.lock().unwrap().console =
                                                    Console::with_cartridge(&bytes);
                                            },
                                        );
                                    }
                                    Sram => {
                                        read_file("SRAM data", &["bin"]).map(|bytes| {
                                            emu.lock().unwrap().load_sram(&bytes);
                                        });
                                    }
                                    Savestate => {
                                        read_file("Super Y.A.N.E Savestate", &["bin"]).map(
                                            |bytes| {
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
                                                };
                                            },
                                        );
                                    }
                                },
                                Save(src) => match src {
                                    Sram => {
                                        let data =
                                            emu.lock().unwrap().console.cartridge().sram.clone();
                                        write_file("SRAM data", &["bin"], &data);
                                    }
                                    Savestate => {
                                        let data = {
                                            serde_brief::to_vec(&emu.lock().unwrap().console)
                                                .unwrap()
                                        };
                                        write_file("Super Y.A.N.E. Savestate", &["bin"], &data);
                                    }
                                    other => {
                                        log::error!("Invalid save src: {:?}", other);
                                    }
                                },
                                OpenTab(tab) => tree
                                    .lock()
                                    .unwrap()
                                    .push_to_focused_leaf((Id::new(SystemTime::now()), tab)),
                                OpenSettings => {
                                    tree.lock().unwrap().add_window(vec![(
                                        Id::new(SystemTime::now()),
                                        EmuTab::Settings,
                                    )]);
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
