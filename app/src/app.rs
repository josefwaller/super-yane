use eframe::CreationContext;
use egui::{Color32, ColorImage, TextureHandle, Ui};
use egui_dock::{DockArea, DockState};
use egui_infinite_scroll::InfiniteScroll;
use super_yane::ppu::SCREEN_RESOLUTION;

use crate::{
    Console, Engine,
    engine::Command,
    ui::{EmuTab, TabViewer, cpu_data},
};

pub struct App {
    engine: Engine,
    screen_data: Option<TextureHandle>,
    tree: DockState<EmuTab>,
    scroll: InfiniteScroll<i32, i32>,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, console: Console) -> Self {
        App {
            engine: Engine::new(console, cc.egui_ctx.clone()),
            screen_data: None,
            tree: DockState::new(vec![
                EmuTab::Screen,
                EmuTab::Cpu,
                EmuTab::Controls,
                EmuTab::CpuDisassembly,
            ]),
            scroll: InfiniteScroll::new().end_loader(|cursor, callback| {
                let start = cursor.unwrap_or(0);
                let end = 1;
                callback(Ok(((start..end).collect(), Some(end))));
            }),
        }
    }
    fn initialize_texture(ui: &mut Ui) -> TextureHandle {
        ui.load_texture(
            "screen_data",
            ColorImage::filled(SCREEN_RESOLUTION, Color32::RED),
            Default::default(),
        )
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Get lock on console
        let emu_arc = self.engine.emulation.clone();

        // Update screen data
        let tex = self
            .screen_data
            .get_or_insert_with(|| App::initialize_texture(ui));
        {
            let emu = emu_arc.lock().expect("Unable to get a lock on emulation");
            tex.set(
                ColorImage::from_rgb(SCREEN_RESOLUTION, &emu.screen_data_rgb),
                Default::default(),
            );
        }
        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_inside(
                ui,
                &mut TabViewer {
                    engine: &mut self.engine,
                    screen: tex,
                    scroll: &mut self.scroll,
                },
            );
    }
}
