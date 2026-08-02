use egui::{Context as UiContext, Key};
use gilrs::Gilrs;
use strum::EnumIter;
use super_yane::{Console, InputPort, ppu::SCREEN_RESOLUTION};
use wdc65816::opcodes::{STP, WDM};

use crate::{
    apu_snapshot::ApuSnapshot,
    cpu_snapshot::CpuSnapshot,
    disassembler::{ApuInstruction, CpuInstruction, Disassembler},
    engine::{AdvanceAmount, Command},
    keybindings::{Input, Keybindings, get_default_keyboard_keybindings},
};

#[derive(EnumIter, Debug, Clone, PartialEq, Copy)]
pub enum Breakpoint {
    Pc(usize),
    Opcode(u8),
    Dma(usize),
}

impl Breakpoint {
    pub fn type_name(&self) -> &'static str {
        use Breakpoint::*;
        match self {
            Pc(_) => "PC",
            Opcode(_) => "Opcode",
            Dma(_) => "DMA Transfer",
        }
    }
}

fn input_pressed(input: Input, ctx: &egui::Context, gilrs: &Gilrs) -> bool {
    match input {
        Input::Key(k) => ctx.input(|i| i.key_pressed(k)),
        Input::Gamepad(id, button) => {
            if gilrs.gamepad(id).is_pressed(button) {
                return true;
            }
            return false;
        }
    }
}

/// The actual data for the emulation thread.
/// Everything here is stored in an Arc<Mutex<>> so that it can be shared
/// between the main thread and the emuation thread.
/// This is intended to run on the emulation thread.
pub struct Emulation {
    /// The console state
    pub console: Console,
    /// The current screen data. This is usually set at the beginning of every VBlank,
    /// but is also set after advancing a set amount
    pub screen_data_rgb: [u8; 3 * SCREEN_RESOLUTION[1] * SCREEN_RESOLUTION[0]],
    pub is_paused: bool,
    pub log_apu: bool,
    pub log_cpu: bool,
    pub volume: f32,
    pub cpu_dis: Disassembler<CpuInstruction>,
    pub apu_dis: Disassembler<ApuInstruction>,
    /// The UI context, used to get input and trigger repaint.
    pub ui_ctx: UiContext,
    /// All breakpoints
    pub breakpoints: Vec<Breakpoint>,
    /// The user set keybingsins
    pub keybindings: Keybindings,
    gilrs: Gilrs,
}

impl Emulation {
    pub fn new(console: Console, ui_ctx: UiContext) -> Emulation {
        let gilrs = Gilrs::new().unwrap();
        Emulation {
            console,
            screen_data_rgb: [0; 3 * SCREEN_RESOLUTION[1] * SCREEN_RESOLUTION[0]],
            volume: 5.0,
            is_paused: false,
            log_apu: false,
            log_cpu: false,
            cpu_dis: Disassembler::<CpuInstruction>::new(),
            apu_dis: Disassembler::<ApuInstruction>::new(),
            ui_ctx,
            // Default breakpoints
            breakpoints: vec![Breakpoint::Opcode(WDM), Breakpoint::Opcode(STP)],
            keybindings: get_default_keyboard_keybindings(),
            gilrs,
        }
    }
    /// Pre-advance hook, should be called before calling advance a bunch of times.
    /// Sets up input ports.
    pub fn pre_advance(&mut self) {
        // Update GILRS
        self.gilrs.inc();
        while let Some(_) = self.gilrs.next_event() {}
        *self.console.input_ports_mut() = self.get_input_ports();
    }
    /// Advances the console 1 instruction.
    /// Handles disassembly, profiling, logging, etc
    pub fn advance(&mut self) {
        let c = &mut self.console;
        let pc = c.pc();
        // let before_master_cycles = *c.total_master_clocks();
        c.step_cpu();
        self.cpu_dis.add_current_instruction(&c);
        if self.log_cpu && c.pc() != pc {
            let inst = CpuSnapshot::from(&c);
            log::info!("[CPU] {}", inst);
        }
        while c.apu_is_behind() {
            c.step_apu();
            self.apu_dis.add_current_instruction(&c);
            if self.log_apu {
                let inst = ApuSnapshot::from(&c);
                log::info!("[APU] {}", inst);
            }
        }
        // Pause if we have hit a breakpoint
        if self.is_in_breakpoint() {
            self.is_paused = true;
        }
        // profiler.add_current_state(&console, before_master_cycles);
    }
    fn is_in_breakpoint(&self) -> bool {
        use Breakpoint::*;
        self.breakpoints.iter().any(|b| match b {
            Dma(index) => self.console.dma_channels()[*index].is_executing,
            Pc(pc) => self.console.pc() == *pc,
            _ => false,
        })
    }
    /// Derives the input port state from the current keyboard/mouse state
    fn get_input_ports(&self) -> [InputPort; 2] {
        [InputPort::StandardController {
            a: input_pressed(self.keybindings.a, &self.ui_ctx, &self.gilrs),
            b: input_pressed(self.keybindings.b, &self.ui_ctx, &self.gilrs),
            x: input_pressed(self.keybindings.x, &self.ui_ctx, &self.gilrs),
            y: input_pressed(self.keybindings.y, &self.ui_ctx, &self.gilrs),
            up: input_pressed(self.keybindings.up, &self.ui_ctx, &self.gilrs),
            left: input_pressed(self.keybindings.left, &self.ui_ctx, &self.gilrs),
            right: input_pressed(self.keybindings.right, &self.ui_ctx, &self.gilrs),
            down: input_pressed(self.keybindings.down, &self.ui_ctx, &self.gilrs),
            start: input_pressed(self.keybindings.start, &self.ui_ctx, &self.gilrs),
            select: input_pressed(self.keybindings.select, &self.ui_ctx, &self.gilrs),
            r: input_pressed(self.keybindings.r, &self.ui_ctx, &self.gilrs),
            l: input_pressed(self.keybindings.l, &self.ui_ctx, &self.gilrs),
        }; 2]
    }
    /// Updates screen data after advancing.
    pub fn post_advance(&mut self) {
        // Copy screen RBG value
        let data = self.console.ppu().screen_data_rgb().as_flattened();
        self.screen_data_rgb[0..data.len()].copy_from_slice(data);
        // Trigger UI refresh
        self.ui_ctx.request_repaint();
    }

    pub fn on_command(&mut self, command: Command) {
        use Command::*;
        match command {
            Advance(a) => {
                self.pre_advance();
                use AdvanceAmount::*;
                match a {
                    MasterCycles(n) => {
                        let goal_cycles = self.console.total_master_clocks() + n as u64;
                        while *self.console.total_master_clocks() < goal_cycles {
                            self.advance();
                        }
                    }
                    Scanlines(n) => (0..n).for_each(|_| {
                        let mut hblank = self.console.ppu().is_in_hblank();
                        while !(hblank && !self.console.ppu().is_in_hblank()) {
                            hblank = self.console.ppu().is_in_hblank();
                            self.advance();
                        }
                    }),
                    Instructions(instructions) => {
                        (0..instructions).for_each(|_| {
                            self.advance();
                        });
                    }
                    Frames(n) => (0..n).for_each(|_| {
                        let mut v = self.console.ppu().is_in_vblank();
                        while !(!v && self.console.ppu().is_in_vblank()) {
                            v = self.console.ppu().is_in_vblank();
                            self.advance();
                        }
                    }),
                    StartVBlank => {
                        let mut vblank = self.console.ppu().is_in_vblank();
                        while !(!vblank && self.console.ppu().is_in_vblank()) {
                            vblank = self.console.ppu().is_in_vblank();
                            self.advance();
                        }
                    }
                    EndVBlank => {
                        let mut vblank = self.console.ppu().is_in_vblank();
                        while !(vblank && !self.console.ppu().is_in_vblank()) {
                            vblank = self.console.ppu().is_in_vblank();
                            self.advance();
                        }
                    }
                }
                self.post_advance();
            }
            LoadRom(bytes) => {
                self.console = Console::with_cartridge(&bytes);
            }
            LoadSavestate(state) => {}
            Reset => {
                self.console.reset();
            }
        };
    }
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.console = Console::with_cartridge(rom);
    }
    pub fn load_savestate(&mut self, state: Console) {
        self.console = state;
        self.console.ppu_mut().reset_vram_cache();
    }
    pub fn load_sram(&mut self, sram: &[u8]) {
        self.console.cartridge_mut().sram = sram.to_owned();
        self.console.reset();
    }
}
