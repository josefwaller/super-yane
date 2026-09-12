use egui::Ui;

use crate::{
    emulation::Emulation,
    ui::{
        colors::RED_PRIMARY,
        widgets::key_value_tree::{KvNode, value_tree},
    },
};

pub fn ppu_data(ui: &mut Ui, emu: &Emulation) {
    let ppu = &emu.console.ppu();
    value_tree(
        ui,
        &KvNode::with_children(
            "PPU",
            "",
            &[
                KvNode::new("Dot", format!("{:?}", ppu.dot_xy())),
                KvNode::new("VBlank Flag", format!("{}", ppu.vblank)),
                KvNode::new("Forced Blanking", format!("{}", ppu.forced_blanking)),
                KvNode::new("Brightness", format!("{:02X}", ppu.brightness)),
                KvNode::new("BG Mode", format!("{:02X}", ppu.bg_mode)),
                KvNode::new("BG3 Priority", format!("{}", ppu.bg3_prio)),
                KvNode::new("Mosaic Size", format!("{:02X}", ppu.mosaic_size)),
                KvNode::new("VRAM Address", format!("{:02X}", ppu.vram_addr)),
                KvNode::new("VRAM Word Address", format!("{:03X}", 2 * ppu.vram_addr)),
                KvNode::new("VRAM INC AMT", format!("{:02X}", ppu.vram_increment_amount)),
                KvNode::new("VRAM INC MODE", format!("{}", ppu.vram_increment_mode)),
                KvNode::new("VRAM Remap", format!("{:02X}", ppu.vram_remap)),
                KvNode::new("CGRAM Address", format!("{:02X}", ppu.cgram_addr)),
                KvNode::new("OBJ Enable Main", format!("{}", ppu.obj_main_enable)),
                KvNode::new("OBJ Enable Sub", format!("{}", ppu.obj_subscreen_enable)),
                KvNode::with_children(
                    "Mode 7",
                    "",
                    &[KvNode::new(
                        "Matrix",
                        format!("{:02X?}", ppu.matrix.as_array()),
                    )],
                ),
                KvNode::new(
                    "Window OBJ Enable Main",
                    format!("{}", ppu.windows_enabled_obj_main),
                ),
                KvNode::new(
                    "Window OBJ Enable Sub",
                    format!("{}", ppu.windows_enabled_obj_sub),
                ),
                KvNode::with_children(
                    "OAM",
                    "",
                    &[
                        KvNode::new("Sizes", format!("{:?}", ppu.oam_sizes)),
                        KvNode::new("Name Address", format!("{:04X}", ppu.oam_name_addr)),
                        KvNode::new("Name Select", format!("{:04X}", ppu.oam_name_select)),
                    ],
                ),
                KvNode::new("Fixed color", format!("{:04X?}", ppu.fixed_color)),
                KvNode::with_children(
                    "Color Math",
                    "",
                    &[
                        KvNode::new("Source", format!("{:?}", ppu.color_math_src)),
                        KvNode::new(
                            "Enabled Backdrop",
                            format!("{}", ppu.color_math_enable_backdrop),
                        ),
                        KvNode::new("Enabled Sprites", format!("{}", ppu.color_math_enable_obj)),
                        KvNode::new("Blend Mode", format!("{:?}", ppu.color_blend_mode)),
                        KvNode::new("Main Region", format!("{:?}", ppu.color_window_main_region)),
                        KvNode::new("Sub Region", format!("{:?}", ppu.color_window_sub_region)),
                    ],
                ),
                KvNode::with_children(
                    "HV Timer",
                    "",
                    &[
                        KvNode::new("Mode", format!("{:?}", ppu.timer_mode)),
                        KvNode::new("H Timer", format!("{:03X}", ppu.h_timer)),
                        KvNode::new("V Timer", format!("{:03X}", ppu.v_timer)),
                    ],
                ),
            ],
        ),
        0,
        &[RED_PRIMARY],
    );
}
