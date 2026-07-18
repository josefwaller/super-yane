use egui::TextureHandle;
use egui_dock::NodePath;
use strum::{EnumIter, EnumString, IntoEnumIterator};

use crate::{
    engine::{Command, Engine},
    ui::{
        binary_table::binary_table,
        breakpoints::breakpoints,
        colors::{COLOR_ORANGE, GREEN_PRIMARY, LIGHT_BLUE_PRIMARY, RED_PRIMARY},
        cpu_data, cpu_disassembly, dma_channels, ppu_data, screen,
    },
};

#[derive(EnumIter, EnumString, Copy, Clone)]
pub enum EmuTab {
    Screen,
    Cpu,
    Ppu,
    DmaChannels,
    CpuDisassembly,
    CpuBreakpoints,
    Wram,
    Vram,
    Cgram,
    Aram,
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
            CpuBreakpoints => "CPU Breakpoints",
            Wram => "WRAM",
            Vram => "VRAM",
            Cgram => "CGRAM",
            Aram => "ARAM",
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
                CpuBreakpoints => breakpoints(ui, &mut emu),
                Wram => binary_table(ui, emu.console.ram().as_slice(), LIGHT_BLUE_PRIMARY),
                Vram => binary_table(ui, emu.console.ppu().vram.as_slice(), RED_PRIMARY),
                Cgram => binary_table(
                    ui,
                    emu.console
                        .ppu()
                        .cgram
                        .map(|f| f.to_le_bytes())
                        .as_flattened(),
                    COLOR_ORANGE,
                ),
                Aram => binary_table(ui, emu.console.apu().ram(), GREEN_PRIMARY),
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
