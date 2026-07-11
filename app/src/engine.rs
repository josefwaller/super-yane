// use crate::{
//     AppWindow, DisassemblyLine, OamData,
//     utils::{get_oam_data, update_binary_data},
// };
use closure::closure;
use derive_new::new;
use egui::{Context as UiContext, Key};
use log::*;
use slint::{Image, Model, ModelRc, Rgb8Pixel, SharedPixelBuffer, VecModel, Weak};
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self},
    time::{Duration, Instant},
};
use super_yane::{Console, Cpu, InputPort, MASTER_CLOCK_SPEED_HZ, Ppu, ppu::SCREEN_RESOLUTION};

const SLEEP_TIME: Duration = Duration::from_millis(5);

use crate::{
    // ConsoleData,
    apu_snapshot::ApuSnapshot,
    audio::Audio,
    cpu_snapshot::CpuSnapshot,
    disassembler::{ApuInstruction, CpuInstruction, Disassembler, Instruction},
    profiler::Profiler,
};

#[derive(Copy, Clone)]
pub struct EmulationContext {}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceAmount {
    MasterCycles(u32),
    Scanlines(u32),
    Instructions(u32),
    Frames(u32),
    StartVBlank,
    EndVBlank,
}

/// Command send to the emulation thread
pub enum Command {
    Advance(AdvanceAmount),
    UpdateInputPorts([InputPort; 2]),
    LoadRom(Vec<u8>),
    LoadSavestate(Console),
    Reset,
}
/// The payload send to the emulation thread telling it to update the emulator
#[derive(new)]
pub struct UpdateEmuPayload {
    /// How much to advance the emulator by
    command: Command,
}

/// The actual data for the emulation thread.
/// Everything here is stored in an Arc<Mutex<>> so that it can be shared
/// between the main thread and the emuation thread
pub struct Emulation {
    /// The console state
    pub console: Console,
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
    /// Pre-advance hook, should be called before calling advance a bunch of times.
    /// Sets up input ports.
    fn pre_advance(&mut self) {
        *self.console.input_ports_mut() = self.get_input_ports();
    }
    /// Advances the console 1 instruction.
    /// Handles disassembly, profiling, logging, etc
    fn advance(&mut self) {
        let c = &mut self.console;
        let pc = c.pc();
        // let before_master_cycles = *c.total_master_clocks();
        c.step_cpu();
        self.cpu_dis.add_current_instruction(&c);
        if self.log_cpu && c.pc() != pc {
            let inst = CpuSnapshot::from(&c);
            info!("[CPU] {}", inst);
        }
        while c.apu_is_behind() {
            c.step_apu();
            self.apu_dis.add_current_instruction(&c);
            if self.log_apu {
                let inst = ApuSnapshot::from(&c);
                info!("[APU] {}", inst);
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
            }
            // UpdateInputPorts(input_ports) => {
            //     *self.console.input_ports_mut() = input_ports;
            // }
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

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    to_emu: Sender<UpdateEmuPayload>,
    pub emulation: Arc<Mutex<Emulation>>,
}

impl Engine {
    pub fn new(
        console: Console, //     , ui_ptr: Weak<AppWindow>
        ui_ctx: UiContext,
    ) -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (to_emu, from_main) = mpsc::channel::<UpdateEmuPayload>();
        // Initialize audio
        let mut audio = Audio::new();
        let emulation = Arc::new(Mutex::new(Emulation {
            console,
            volume: 20.0,
            is_paused: false,
            log_apu: false,
            log_cpu: false,
            cpu_dis: Disassembler::<CpuInstruction>::new(),
            apu_dis: Disassembler::<ApuInstruction>::new(),
            ui_ctx,
        }));

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(closure!(clone emulation,
                //, clone ui_ptr,
                 || {
                {
                    let mut e = emulation.lock().unwrap();
                    let c = e.console.clone();
                    // Add initial vectors and instruction
                    e.cpu_dis.add_native_vectors(&c);
                    e.cpu_dis.add_current_instruction(&c);
                    e.apu_dis.add_current_instruction(&c);
                }

                // Used to calculate delta time to advance the emulator
                let mut last_time = Instant::now();
                loop {
                       // Send data back to the main thread for slint to display
                        let p = from_main.try_recv();
                        match p {
                            Ok(payload) => {
                                emulation.lock().unwrap().on_command(payload.command);
                                // update_ui(emulation.clone(), ui_ptr.clone());
                            },
                            Err(_) => {}
                        }
                        // Calculate delta time
                        let now = Instant::now();
                        let dt = now - last_time;
                        last_time = now;
                        {
                        let mut e = emulation.lock().unwrap();
                        // Advance emulator
                        if !e.is_paused {
                            e.pre_advance();
                            let initial_master_cycles = e.console.total_master_clocks().clone();
                            while ((e.console.total_master_clocks() - initial_master_cycles) as f64)
                                < dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64
                            {
                                let vblank = e.console.ppu().is_in_vblank();
                                e.advance();
                                // Update canvas if we just entered vblank
                                if !vblank && e.console.ppu().is_in_vblank() {
                                    // update_ui(emulation.clone(), ui_ptr.clone());
                                }
                            }
                            // Update audio
                            let samples = e.console.apu_mut().sample_queue();
                            let (a, b) = samples.as_slices();
                            audio.push_samples(a, e.volume);
                            audio.push_samples(b, e.volume);

                        }
                        // Repaint
                        e.ui_ctx.request_repaint();
                    }
                        // Sleep
                        thread::sleep(SLEEP_TIME);
                    }
            }))
            .expect("Unable to spawn thread");

        // Set the initial settings
        // ui_ptr
        //     .upgrade()
        //     .unwrap()
        //     .set_settings(emulation.lock().unwrap().settings.clone());
        Engine { to_emu, emulation }
    }

    pub fn update(&mut self, command: Command) {
        self.to_emu
            .send(UpdateEmuPayload { command })
            .expect("Unable to send data to thread");
    }

    pub fn get_savestate(&self) -> Vec<u8> {
        let c = self.emulation.lock().unwrap().console.clone();
        serde_brief::to_vec::<Console>(&c).expect("Unable to serialize console")
    }
    pub fn load_savestate(&mut self, state: &[u8]) -> Result<(), serde_brief::Error> {
        let c: Console = serde_brief::from_slice(state)?;
        self.to_emu
            .send(UpdateEmuPayload::new(Command::LoadSavestate(c)))
            .unwrap();
        Ok(())
    }
}
