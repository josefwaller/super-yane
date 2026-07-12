use egui::TextureHandle;
use super_yane::Console;

use crate::{
    engine::{Command, Engine},
    ui::cpu_data,
};

pub enum Tab {
    Screen,
    Cpu,
    Controls,
}
pub struct TabViewer<'a> {
    pub engine: &'a mut Engine,
    pub screen: &'a TextureHandle,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Tab;
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        egui::WidgetText::Text(
            match tab {
                Tab::Cpu => "WDC 65816",
                Tab::Screen => "Screen",
                Tab::Controls => "Controls",
            }
            .to_owned(),
        )
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Move command out to respect rust memory safety rules
        let mut to_send: Option<Command> = None;
        {
            let mut emu = self
                .engine
                .emulation
                .lock()
                .expect("Unable to get a lock on Emulation");
            match tab {
                Tab::Cpu => {
                    cpu_data(ui, &emu.console);
                }
                Tab::Screen => {
                    ui.vertical_centered(|ui| {
                        ui.label(emu.console.cartridge().title());
                        // Render screen
                        let img =
                            egui::Image::from_texture((self.screen.id(), self.screen.size_vec2()))
                                .max_size(ui.available_size())
                                .fit_to_exact_size(ui.available_size());
                        ui.add(img);
                    });
                }
                Tab::Controls => {
                    ui.horizontal(|ui| {
                        if ui
                            .button(if emu.is_paused { "Resume" } else { "Pause" })
                            .clicked()
                        {
                            emu.is_paused = !emu.is_paused;
                        }
                        if ui.button("Reset").clicked() {
                            to_send = Some(Command::Reset);
                        }
                        let mut vol = emu.volume;
                        ui.add(egui::Slider::new(&mut vol, 0.0..=100.0).text("Volume"));
                        emu.volume = vol;
                    });
                }
            }
        }
        if let Some(command) = to_send {
            self.engine.update(command);
        }
    }
}
