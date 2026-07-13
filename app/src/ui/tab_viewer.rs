use egui::{
    Align, Color32, Layout, RichText, TextureHandle, UiKind::ScrollArea, hex_color,
    style::ScrollAnimation,
};
use egui_dock::{DockState, NodePath};
use egui_infinite_scroll::InfiniteScroll;
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    disassembler::{CpuInstruction, Instruction},
    engine::{AdvanceAmount, Command, Engine},
    ui::cpu_data,
};

#[derive(EnumIter, Copy, Clone)]
pub enum EmuTab {
    Screen,
    Cpu,
    CpuDisassembly,
    BinaryData,
}
impl ToString for EmuTab {
    fn to_string(&self) -> String {
        use EmuTab::*;
        match self {
            Screen => "Screen",
            Cpu => "WDC65816",
            CpuDisassembly => "CPU Instructions",
            BinaryData => "Binary",
        }
        .to_owned()
    }
}
pub struct TabViewer<'a> {
    pub engine: &'a mut Engine,
    pub screen: &'a TextureHandle,
    pub scroll: &'a mut InfiniteScroll<i32, i32>,
    pub added_tabs: &'a mut Vec<(NodePath, EmuTab)>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = (egui::Id, EmuTab);
    fn title(&mut self, (_, tab): &mut Self::Tab) -> egui::WidgetText {
        egui::WidgetText::Text(tab.to_string())
    }
    fn id(&mut self, (id, _): &mut Self::Tab) -> egui::Id {
        *id
    }
    fn ui(&mut self, ui: &mut egui::Ui, (_, tab): &mut Self::Tab) {
        // Move command out to respect rust memory safety rules
        let mut to_send: Option<Command> = None;
        {
            let mut emu = self
                .engine
                .emulation
                .lock()
                .expect("Unable to get a lock on Emulation");
            use EmuTab::*;
            match tab {
                Cpu => {
                    cpu_data(ui, &emu.console);
                }
                Screen => {
                    ui.vertical_centered(|ui| {
                        ui.label(emu.console.cartridge().title());
                        // Render screen
                        let img =
                            egui::Image::from_texture((self.screen.id(), self.screen.size_vec2()))
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
                                if ui.button("Advance").clicked() {
                                    to_send =
                                        Some(Command::Advance(AdvanceAmount::Instructions(1)));
                                }
                                if ui.button("Reset").clicked() {
                                    to_send = Some(Command::Reset);
                                }
                                let mut vol = emu.volume;
                                ui.add(egui::Slider::new(&mut vol, 0.0..=100.0).text("Volume"));
                                emu.volume = vol;
                            });
                        });
                    });
                }
                CpuDisassembly => {
                    let mut scroll = egui::ScrollArea::vertical().hscroll(emu.is_paused);
                    let pc = emu.console.cartridge().transform_address(emu.console.pc());
                    // Get height of each row (since they're just text, it's just hte text height)
                    let height = ui.text_style_height(&egui::TextStyle::Body);
                    if !emu.is_paused {
                        // Scroll to current instruction
                        let index = {
                            let inst = CpuInstruction::current_instruction(&emu.console);
                            emu.cpu_dis
                                .instructions()
                                .keys()
                                .position(|k| *k == inst.key())
                        };
                        if let Some(i) = index {
                            // Compute the scroll offset
                            scroll = scroll
                                .vertical_scroll_offset(
                                    (height + ui.spacing().item_spacing.y) * i as f32,
                                )
                                .animated(false);
                        }
                    }
                    scroll.show_rows(
                        ui,
                        height,
                        emu.cpu_dis.instructions().len(),
                        |ui, row_range| {
                            for index in row_range {
                                ui.set_width(ui.available_width());
                                // self.scroll.ui(ui, 10, |ui, index, item| {
                                let line = emu.cpu_dis.lines().nth(index);
                                match line {
                                    None => {}
                                    Some(l) => {
                                        let is_current_inst = if l.pc == pc { true } else { false };
                                        ui.columns(4, |cols| {
                                            if is_current_inst {
                                                cols[0].label(
                                                    RichText::new("->").color(hex_color!("FF0000")),
                                                );
                                                cols[0].set_max_width(20.0);
                                            }
                                            if let Some(label) = l.label {
                                                cols[1].label(label.to_string());
                                            }
                                            cols[2].label(format!("{:06X}", l.pc));
                                            cols[3].label(RichText::new(
                                                l.instruction.to_string(emu.cpu_dis.labels()),
                                            ));
                                        })
                                    }
                                }
                            }
                            // });
                        },
                    );
                }
                BinaryData => {
                    ui.label("Binary Data");
                }
            }
        }
        if let Some(command) = to_send {
            self.engine.update(command);
        }
    }
    fn add_popup(&mut self, ui: &mut egui::Ui, path: egui_dock::NodePath) {
        ui.vertical(|ui| {
            EmuTab::iter().for_each(|tab| {
                if ui.button(tab.to_string()).clicked() {
                    self.added_tabs.push((path, tab));
                }
            });
        });
    }
}
