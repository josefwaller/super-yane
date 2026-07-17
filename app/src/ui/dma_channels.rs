use std::sync::Arc;

use egui::{Align, Layout, Ui, mutex::Mutex};
use super_yane::dma::TRANSFER_PATTERNS;

use crate::emulation::Emulation;

pub fn dma_channels(ui: &mut Ui, emu: &Emulation) {
    emu.console
        .dma_channels()
        .iter()
        .enumerate()
        .for_each(|(i, c)| {
            ui.vertical(|ui| {
                ui.label(format!("Channel {}", i));
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.with_layout(
                        Layout::top_down(Align::Min).with_cross_justify(false),
                        |ui| {
                            macro_rules! row {
                                ($name: expr, $value: expr, $fmt: expr) => {
                                    ui.columns(2, |cols| {
                                        cols[0].label($name);
                                        cols[1].label(format!($fmt, $value));
                                    })
                                };
                            }
                            row!("HDMA", c.is_hdma(), "{}");
                            row!("Source", c.full_src_addr(), "{:06X}");
                            row!("Destination", c.dest_addr, "{:04X}");
                            row!("Indirect", c.indirect, "{}");
                            row!("Byte Counter", c.byte_counter, "{:04X}");
                            row!(
                                "Transfer Pattern",
                                TRANSFER_PATTERNS[c.transfer_pattern_index],
                                "{:?}"
                            );
                            row!("Address Adjust Mode", c.adjust_mode, "{:?}");
                            row!("Direction", c.direction, "{}");
                            row!("Line Counter", c.hdma_line_counter, "{:02X}");
                            row!("HDMA Table Address", c.current_hdma_table_addr(0), "{:06X}");
                            row!("HDMA Repeat", c.hdma_repeat, "{}");
                        },
                    )
                })
            });
        });
}
