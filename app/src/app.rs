use eframe::CreationContext;
use egui::{Color32, ColorImage, TextureHandle, Ui};
use super_yane::ppu::SCREEN_RESOLUTION;

use crate::{Command, Console, Engine};

pub struct App {
    engine: Engine,
    screen_data: Option<TextureHandle>,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, console: Console) -> Self {
        App {
            engine: Engine::new(console, cc.egui_ctx.clone()),
            screen_data: None,
        }
    }
    pub fn initialize_texture(ui: &mut Ui) -> TextureHandle {
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
        let lock = self.engine.emulation.clone();
        let emu = lock.lock().expect("Unable to get a lock on emulation");
        // Update screen data
        let tex = self
            .screen_data
            .get_or_insert_with(|| App::initialize_texture(ui));
        tex.set(
            ColorImage::from_rgb(
                SCREEN_RESOLUTION,
                emu.console.ppu().screen_data_rgb().as_flattened(),
            ),
            Default::default(),
        );
        // Render screen
        ui.image((tex.id(), tex.size_vec2()));
    }
}
