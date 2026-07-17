use egui::{Label, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{emulation::Emulation, ui::colors::COLOR_RED};

pub fn ppu_data(ui: &mut Ui, emu: &Emulation) {
    let ppu = &emu.console.ppu();
    let row_data: &[(&str, String)] = &[
        ("Dot", format!("{:?}", ppu.dot_xy())),
        ("VBlank Flag", format!("{}", ppu.vblank)),
        ("Forced Blanking", format!("{}", ppu.forced_blanking)),
        ("Brightness", format!("{:02X}", ppu.brightness)),
        ("BG Mode", format!("{:02X}", ppu.bg_mode)),
        ("BG3 Priority", format!("{}", ppu.bg3_prio)),
        ("Mosaic Size", format!("{:02X}", ppu.mosaic_size)),
        ("VRAM Address", format!("{:02X}", ppu.vram_addr)),
        ("VRAM INC AMT", format!("{:02X}", ppu.vram_increment_amount)),
        ("VRAM INC MODE", format!("{}", ppu.vram_increment_mode)),
        ("VRAM Remap", format!("{:02X}", ppu.vram_remap)),
        ("CGRAM Address", format!("{:02X}", ppu.cgram_addr)),
        ("OBJ Enable Main", format!("{}", ppu.obj_main_enable)),
        ("OBJ Enable Sub", format!("{}", ppu.obj_subscreen_enable)),
        (
            "Window OBJ Enable Main",
            format!("{}", ppu.windows_enabled_obj_main),
        ),
        (
            "Window OBJ Enable Sub",
            format!("{}", ppu.windows_enabled_obj_sub),
        ),
        ("OAM Sizes", format!("{:?}", ppu.oam_sizes)),
        ("OAM Name Address", format!("{:04X}", ppu.oam_name_addr)),
        ("OAM Name Select", format!("{:04X}", ppu.oam_name_select)),
    ];
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .body(|body| {
            body.rows(18.0, row_data.len(), |mut row| {
                let index = row.index();
                let (left, right) = row_data[index].clone();
                row.col(|ui| {
                    let label = Label::new(RichText::new(left.to_owned()).color(COLOR_RED))
                        .wrap_mode(egui::TextWrapMode::Extend);
                    ui.add(label);
                });
                row.col(|ui| {
                    let label = Label::new(right).wrap_mode(egui::TextWrapMode::Extend);
                    ui.add(label);
                });
            });
        });
}
