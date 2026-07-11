use egui::{Context as UiContext, Key};
use super_yane::{Console, InputPort, ppu::SCREEN_RESOLUTION};

use crate::{
    apu_snapshot::ApuSnapshot,
    cpu_snapshot::CpuSnapshot,
    disassembler::{ApuInstruction, CpuInstruction, Disassembler},
    engine::{AdvanceAmount, Command},
};

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
}

impl Emulation {
    pub fn new(console: Console, ui_ctx: UiContext) -> Emulation {
        Emulation {
            console,
            screen_data_rgb: [0; 3 * SCREEN_RESOLUTION[1] * SCREEN_RESOLUTION[0]],
            volume: 20.0,
            is_paused: false,
            log_apu: false,
            log_cpu: false,
            cpu_dis: Disassembler::<CpuInstruction>::new(),
            apu_dis: Disassembler::<ApuInstruction>::new(),
            ui_ctx,
        }
    }
    /// Pre-advance hook, should be called before calling advance a bunch of times.
    /// Sets up input ports.
    pub fn pre_advance(&mut self) {
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
        // profiler.add_current_state(&console, before_master_cycles);
    }
    /// Derives the input port state from the current keyboard/mouse state
    fn get_input_ports(&self) -> [InputPort; 2] {
        self.ui_ctx.input(|i| {
            // TODO: Use custom keybindings here
            [InputPort::StandardController {
                a: i.key_down(Key::B),
                b: i.key_down(Key::Space),
                x: i.key_down(Key::N),
                y: i.key_down(Key::M),
                up: i.key_down(Key::W),
                left: i.key_down(Key::A),
                right: i.key_down(Key::D),
                down: i.key_down(Key::S),
                start: i.key_down(Key::R),
                select: i.key_down(Key::F),
                r: i.key_down(Key::E),
                l: i.key_down(Key::Q),
            }; 2]
        })
    }
    /// Updates screen data after advancing.
    pub fn post_advance(&mut self) {
        self.screen_data_rgb
            .copy_from_slice(self.console.ppu().screen_data_rgb().as_flattened());
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
            LoadSavestate(state) => {
                self.console = state;
                self.console.ppu_mut().reset_vram_cache();
            }
            Reset => {
                self.console.reset();
            }
            _ => unimplemented!(),
        };
    }
}
