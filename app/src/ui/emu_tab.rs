use egui::Ui;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    app::AppState,
    emulation::Emulation,
    ui::{
        colors::{COLOR_ORANGE, GREEN_PRIMARY, LIGHT_BLUE_PRIMARY, PINK_PRIMARY, RED_PRIMARY},
        panes::{
            apu_data, backgrounds, binary_data, breakpoints, cartridge_data, cpu_data,
            cpu_disassembly, dma_channels, oam, ppu_data, screen, settings,
        },
    },
};

/// All of the different types of tabs that the user can open in the tab viewer
#[derive(Serialize, Deserialize, EnumIter, Copy, Clone)]
pub enum EmuTab {
    Screen,
    Cpu,
    Ppu,
    Apu,
    DmaChannels,
    CpuDisassembly,
    CpuBreakpoints,
    Backgrounds,
    Oam,
    Wram,
    Vram,
    Cgram,
    Aram,
    CartridgeRom,
    CartridgeInfo,
    Settings,
}
impl ToString for EmuTab {
    fn to_string(&self) -> String {
        use EmuTab::*;
        match self {
            Screen => "Emulation",
            Cpu => "WDC65816",
            Ppu => "PPU",
            Apu => "SPC700",
            DmaChannels => "DMA",
            CpuDisassembly => "CPU Instructions",
            CpuBreakpoints => "CPU Breakpoints",
            Backgrounds => "Backgrounds",
            Oam => "OAM",
            Wram => "WRAM",
            Vram => "VRAM",
            Cgram => "CGRAM",
            Aram => "ARAM",
            CartridgeRom => "ROM",
            CartridgeInfo => "Cartridge",
            Settings => "Settings",
        }
        .to_owned()
    }
}
/// Render a given tab
pub fn render_tab_pane(ui: &mut Ui, tab: EmuTab, emu: &mut Emulation, app_state: &mut AppState) {
    use EmuTab::*;
    match tab {
        Cpu => cpu_data(ui, &emu.console),
        Ppu => ppu_data(ui, &emu),
        Apu => apu_data(ui, &emu),
        Screen => screen(ui, emu, app_state),
        DmaChannels => dma_channels(ui, &emu),
        CpuDisassembly => cpu_disassembly(ui, &emu),
        CpuBreakpoints => breakpoints(ui, emu),
        Oam => oam(ui, &emu),
        Backgrounds => backgrounds(ui, &emu),
        Wram => binary_data(
            ui,
            emu.console.ram().as_slice(),
            LIGHT_BLUE_PRIMARY,
            &emu.console.ppu().cgram,
        ),
        Vram => binary_data(
            ui,
            emu.console.ppu().vram.as_slice(),
            RED_PRIMARY,
            &emu.console.ppu().cgram,
        ),
        Cgram => binary_data(
            ui,
            emu.console
                .ppu()
                .cgram
                .map(|f| f.to_le_bytes())
                .as_flattened(),
            COLOR_ORANGE,
            &emu.console.ppu().cgram,
        ),
        Aram => binary_data(
            ui,
            emu.console.apu().ram(),
            GREEN_PRIMARY,
            &emu.console.ppu().cgram,
        ),
        CartridgeRom => binary_data(
            ui,
            &emu.console.cartridge().data,
            PINK_PRIMARY,
            &emu.console.ppu().cgram,
        ),
        CartridgeInfo => cartridge_data(ui, &emu.console),
        Settings => settings(ui, emu),
    }
}
