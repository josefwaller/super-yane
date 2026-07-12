use eframe::CreationContext;
use egui::{Color32, ColorImage, TextureHandle, Ui};
use super_yane::ppu::SCREEN_RESOLUTION;

use crate::{Console, Engine, engine::Command, ui::cpu_data};

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
        let mut emu = emu_arc.lock().expect("Unable to get a lock on emulation");
        egui::containers::Panel::bottom("Bottom")
            .resizable(true)
            .show(ui, |ui| ui.label("Coming soon"));
        egui::containers::Panel::left("Left")
            .resizable(true)
            .show(ui, |ui| {
                cpu_data(ui, &emu.console);
            });
        egui::containers::Panel::right("Right")
            .resizable(true)
            .show(ui, |ui| ui.label("Coming soon"));
        // Update screen data
        let tex = self
            .screen_data
            .get_or_insert_with(|| App::initialize_texture(ui));
        tex.set(
            ColorImage::from_rgb(SCREEN_RESOLUTION, &emu.screen_data_rgb),
            Default::default(),
        );
        ui.vertical(|ui| {
            ui.vertical_centered(|ui| ui.label(emu.console.cartridge().title()));
            // Render screen
            let img = egui::Image::from_texture((tex.id(), tex.size_vec2()))
                .max_size(ui.available_size())
                .fit_to_exact_size(ui.available_size());
            ui.add(img);
            ui.horizontal(|ui| {
                if ui
                    .button(if emu.is_paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    emu.is_paused = !emu.is_paused;
                }
                if ui.button("Reset").clicked() {
                    self.engine.update(Command::Reset);
                }
                let mut vol = emu.volume;
                ui.add(egui::Slider::new(&mut vol, 0.0..=100.0).text("Volume"));
                emu.volume = vol;
            })
        });
    }
}
