use std::sync::Arc;

use egui::{Align, Color32, Layout, Margin, RichText, Ui, mutex::Mutex};
use super_yane::dma::TRANSFER_PATTERNS;

use crate::{
    emulation::Emulation,
    ui::{
        colors::GREEN_PRIMARY,
        vertical_table::{Row, vertical_table},
    },
};

pub fn dma_channels(ui: &mut Ui, emu: &Emulation) {
    const COLOR: Color32 = GREEN_PRIMARY;
    emu.console
        .dma_channels()
        .iter()
        .enumerate()
        .for_each(|(i, c)| {
            ui.vertical(|ui| {
                ui.label(RichText::new(format!("Channel {}", i)).color(COLOR));
                egui::Frame::NONE
                    .inner_margin(Margin {
                        left: 10,
                        ..Margin::ZERO
                    })
                    .show(ui, |ui| {
                        vertical_table(
                            ui,
                            &[
                                Row::new("HDMA", format!("{}", c.is_hdma())),
                                Row::new("Source", format!("{:06X}", c.full_src_addr())),
                                Row::new("Destination", format!("{:04X}", c.dest_addr)),
                                Row::new("Indirect", format!("{}", c.indirect)),
                                Row::new("Byte Counter", format!("{:04X}", c.byte_counter)),
                                Row::new(
                                    "Transfer Pattern",
                                    format!("{:?}", TRANSFER_PATTERNS[c.transfer_pattern_index],),
                                ),
                                Row::new("Address Adjust Mode", format!("{:?}", c.adjust_mode)),
                                Row::new("Direction", format!("{}", c.direction)),
                                Row::new("Line Counter", format!("{:02X}", c.hdma_line_counter)),
                                Row::new(
                                    "HDMA Table Address",
                                    format!("{:06X}", c.current_hdma_table_addr(0)),
                                ),
                                Row::new("HDMA Repeat", format!("{}", c.hdma_repeat)),
                            ],
                            COLOR,
                            format!("DMA {}", i),
                        );
                    });
            });
        });
}
