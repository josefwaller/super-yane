use egui::{Color32, Label, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    emulation::Emulation,
    ui::{
        colors::RED_PRIMARY,
        vertical_table::{Row, vertical_table},
    },
};

pub fn ppu_data(ui: &mut Ui, emu: &Emulation) {
    let ppu = &emu.console.ppu();
    vertical_table(
        ui,
        &[
            Row::new("Dot", format!("{:?}", ppu.dot_xy())),
            Row::new("VBlank Flag", format!("{}", ppu.vblank)),
            Row::new("Forced Blanking", format!("{}", ppu.forced_blanking)),
            Row::new("Brightness", format!("{:02X}", ppu.brightness)),
            Row::new("BG Mode", format!("{:02X}", ppu.bg_mode)),
            Row::new("BG3 Priority", format!("{}", ppu.bg3_prio)),
            Row::new("Mosaic Size", format!("{:02X}", ppu.mosaic_size)),
            Row::new("VRAM Address", format!("{:02X}", ppu.vram_addr)),
            Row::new("VRAM INC AMT", format!("{:02X}", ppu.vram_increment_amount)),
            Row::new("VRAM INC MODE", format!("{}", ppu.vram_increment_mode)),
            Row::new("VRAM Remap", format!("{:02X}", ppu.vram_remap)),
            Row::new("CGRAM Address", format!("{:02X}", ppu.cgram_addr)),
            Row::new("OBJ Enable Main", format!("{}", ppu.obj_main_enable)),
            Row::new("OBJ Enable Sub", format!("{}", ppu.obj_subscreen_enable)),
            Row::new(
                "Window OBJ Enable Main",
                format!("{}", ppu.windows_enabled_obj_main),
            ),
            Row::new(
                "Window OBJ Enable Sub",
                format!("{}", ppu.windows_enabled_obj_sub),
            ),
            Row::new("OAM Sizes", format!("{:?}", ppu.oam_sizes)),
            Row::new("OAM Name Address", format!("{:04X}", ppu.oam_name_addr)),
            Row::new("OAM Name Select", format!("{:04X}", ppu.oam_name_select)),
        ],
        RED_PRIMARY,
        "PPU".to_owned(),
    );
}
