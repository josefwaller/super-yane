use egui::Ui;

use crate::{
    emulation::Emulation,
    ui::{
        colors::{PINK_PRIMARY, RED_PRIMARY, RED_SECONDARY},
        widgets::key_value_tree::{KvNode, value_tree},
    },
};

pub fn backgrounds(ui: &mut Ui, emu: &Emulation) {
    // TODO: Add an "is enabled in this BG mode" flag
    for (i, bg) in emu.console.ppu().backgrounds.iter().enumerate() {
        value_tree(
            ui,
            &KvNode::with_children(
                "Background",
                format!("{}", i),
                &[
                    // Decimal
                    KvNode::new("Tile size", format!("{}", bg.tile_size)),
                    // Decimal
                    KvNode::new("Mosaic Enabled", format!("{}", bg.mosaic)),
                    // Decimal
                    KvNode::new("Horiztonal Tilemaps", format!("{}", bg.num_horz_tilemaps)),
                    // Decimal
                    KvNode::new("Vertical Tilemaps", format!("{}", bg.num_vert_tilemaps)),
                    KvNode::new("Tilemap Address", format!("{:04X}", bg.tilemap_addr)),
                    KvNode::new("Character Address", format!("{:04X}", bg.chr_addr)),
                    KvNode::new("Horizontal Offset", format!("{:02X}", bg.h_off)),
                    KvNode::new("Vertical Offset", format!("{:02X}", bg.v_off)),
                    KvNode::new("Main screen enabled", bg.main_screen_enable),
                    KvNode::new("Sub screen enabled", bg.sub_screen_enable),
                    KvNode::new("Color Math enabled", bg.color_math_enable),
                    KvNode::with_children(
                        "Windows",
                        "",
                        &[
                            KvNode::new("Main screen enabled", bg.windows_enabled_sub),
                            KvNode::new("Sub screen enabled", bg.windows_enabled_sub),
                            KvNode::new("Mask Logic", format!("{:?}", bg.window_mask_logic)),
                            KvNode::with_children(
                                "Window 0",
                                "",
                                &[
                                    KvNode::new("Enabled", bg.window_enabled[0]),
                                    KvNode::new("Inverted", bg.window_invert[0]),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            0,
            &[RED_PRIMARY, RED_SECONDARY],
        );
    }
}
