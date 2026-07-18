use egui::{
    Align, Color32, ComboBox, Layout, RichText, TextureHandle, UiKind::ScrollArea, hex_color,
    style::ScrollAnimation,
};
use egui_dock::{DockState, NodePath};
use egui_infinite_scroll::InfiniteScroll;
use strum::{EnumIter, EnumString, IntoEnumIterator};
use super_yane::dma::TRANSFER_PATTERNS;

use crate::{
    disassembler::{CpuInstruction, Instruction},
    engine::{AdvanceAmount, Command, Engine},
    ui::{cpu_data, cpu_disassembly, dma_channels, ppu_data, screen},
};

#[derive(EnumIter, EnumString, Copy, Clone)]
pub enum EmuTab {
    Screen,
    Cpu,
    Ppu,
    DmaChannels,
    CpuDisassembly,
    BinaryData,
}
impl ToString for EmuTab {
    fn to_string(&self) -> String {
        use EmuTab::*;
        match self {
            Screen => "Emulation",
            Cpu => "WDC65816",
            Ppu => "PPU",
            DmaChannels => "DMA",
            CpuDisassembly => "CPU Instructions",
            BinaryData => "Binary",
        }
        .to_owned()
    }
}
pub struct TabViewer<'a> {
    pub engine: &'a mut Engine,
    pub screen: &'a TextureHandle,
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
                Cpu => cpu_data(ui, &emu.console),
                Ppu => ppu_data(ui, &emu),
                Screen => screen(ui, &mut emu, self.screen, &mut to_send),
                DmaChannels => dma_channels(ui, &emu),
                CpuDisassembly => cpu_disassembly(ui, &emu),
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
