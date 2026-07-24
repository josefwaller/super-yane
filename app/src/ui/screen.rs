use egui::{Layout, Pos2, Rect, TextureHandle, Ui};
use super_yane::ppu::SCREEN_RESOLUTION;

use crate::{
    emulation::Emulation,
    engine::{AdvanceAmount, Command},
};

pub fn screen(
    ui: &mut Ui,
    emu: &mut Emulation,
    screen: &TextureHandle,
    to_send: &mut Option<Command>,
) {
    ui.vertical_centered(|ui| {
        ui.label(emu.console.cartridge().title());
        // Render screen
        let img = egui::Image::from_texture((screen.id(), screen.size_vec2()))
            .uv(Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(
                    1.0,
                    emu.console.ppu().screen_resolution()[1] as f32 / SCREEN_RESOLUTION[1] as f32,
                ),
            ))
            .max_size(ui.available_size())
            .fit_to_exact_size(ui.available_size());
        ui.add(img);
        ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if emu.is_paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    emu.is_paused = !emu.is_paused;
                }
                if ui.button("Step").clicked() {
                    *to_send = Some(Command::Advance(AdvanceAmount::Instructions(1)));
                }
                if ui.button("Start VBlank").clicked() {
                    *to_send = Some(Command::Advance(AdvanceAmount::StartVBlank));
                }
                if ui.button("Scanline").clicked() {
                    *to_send = Some(Command::Advance(AdvanceAmount::Scanlines(1)));
                }
                if ui.button("Reset").clicked() {
                    *to_send = Some(Command::Reset);
                }
                ui.checkbox(&mut emu.log_cpu, "Log CPU");
                let mut vol = emu.volume;
                ui.add(egui::Slider::new(&mut vol, 0.0..=100.0).text("Volume"));
                emu.volume = vol;
            });
        });
    });
}
