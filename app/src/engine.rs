use derive_new::new;
use log::*;
use std::{
    collections::VecDeque,
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self},
    time::{Duration, Instant},
};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ, ppu::SCREEN_RESOLUTION};

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");
const SLEEP_TIME: Duration = Duration::from_millis(5);

use crate::{
    Settings, apu_snapshot::ApuSnapshot, audio::Audio, cpu_snapshot::CpuSnapshot,
    disassembler::Disassembler, profiler::Profiler,
};

/// Misc settings for advancing the emulator
#[derive(Default, Clone)]
pub struct AdvanceSettings {
    /// Whether to log the CPU state after every instruction
    pub log_cpu: bool,
    /// Whether to log the APU state after every instruction
    pub log_apu: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceAmount {
    MasterCycles(u32),
    Scanlines(u32),
    Instructions(u32),
    Frames(u32),
    StartVBlank,
    EndVBlank,
}
/// Adds an 's' to string if n is 0 or n > 1
fn pluralize(string: &str, n: impl Into<u32>) -> String {
    let n = n.into();
    format!("{}{}", string, if n == 0 || n > 1 { "s" } else { "" })
}

impl Display for AdvanceAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AdvanceAmount::*;
        match self {
            MasterCycles(n) => write!(f, "{} {}", n, pluralize("master cycle", *n)),
            Scanlines(n) => write!(f, "{} {}", n, pluralize("scanline", *n)),
            Instructions(n) => write!(f, "{} {}", n, pluralize("instruction", *n)),
            Frames(n) => write!(f, "{} {}", n, pluralize("frame", *n)),
            StartVBlank => write!(f, "Start VBlank"),
            EndVBlank => write!(f, "End VBlank"),
        }
    }
}
/// Command send to the emulation thread
pub enum Command {
    Advance(AdvanceAmount),
    UpdateInputPorts([InputPort; 2]),
    LoadRom(Vec<u8>),
    LoadSavestate(Console),
    SetSettings(Settings),
    Reset,
}
/// The payload send to the emulation thread telling it to update the emulator
#[derive(new)]
pub struct UpdateEmuPayload {
    /// How much to advance the emulator by
    command: Command,
    /// The settings
    settings: AdvanceSettings,
}

/// The Payload to send data back to the main thread after finishing emulation
pub struct DoneEmuPayload {
    console: Console,
    disassembler: Disassembler,
    profiler: Profiler,
}

/// The payload sent every frame from the emulator thread containing output information
#[derive(new)]
pub struct StreamPayload {
    screen_data: [[u8; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
}

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    console: Console,
    pub disassembler: Disassembler,
    pub profiler: Profiler,
    sender: Sender<UpdateEmuPayload>,
    emu_data_receiver: Receiver<DoneEmuPayload>,
    stream_receiver: Receiver<StreamPayload>,
    /// The RGB data from the previous fully rendered frame
    pub prev_frame_data: [[u8; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
}
impl Engine {
    pub fn new() -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (sender, receiver) = mpsc::channel::<UpdateEmuPayload>();
        // Send data back to the main thread after emulating
        let (emu_data_sender, emu_data_receiver) = mpsc::channel::<DoneEmuPayload>();
        // Send new frame data every time a new frame is generated
        let (stream_sender, stream_receiver) = mpsc::channel::<StreamPayload>();
        // Initialize audio
        let mut audio = Audio::new();

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                use Command::*;
                // Create console
                let mut console = Console::with_cartridge(DEFAULT_CARTRIDGE);
                // Used to calculate delta time to advance the emulator
                let mut last_time = Instant::now();
                // The settings
                let mut settings = Settings::default();
                loop {
                    let mut disassembler = Disassembler::new(&console);
                    let mut profiler = Profiler::new();
                    /// Advance by 1 instruction
                    macro_rules! advance {
                        () => {
                            let vblank = console.ppu().is_in_vblank();
                            let before_master_cycles = *console.total_master_clocks();
                            console.step_cpu();
                            // if payload.settings.log_cpu {
                            //     let inst = CpuSnapshot::from(&console);
                            //     info!("{}", inst);
                            // }
                            while console.apu_is_behind() {
                                console.step_apu();
                                // if payload.settings.log_apu {
                                //     let inst = ApuSnapshot::from(&console);
                                //     info!("{}", inst);
                                // }
                            }
                            // disassembler.add_current_instruction(&console);
                            // profiler.add_current_state(&console, before_master_cycles);

                            if vblank && !console.ppu().is_in_vblank() {
                                stream_sender
                                    .send(StreamPayload::new(console.ppu().screen_data_rgb()))
                                    .expect("Unable to send frame data");
                            }
                        };
                    }
                    let p = receiver.try_recv();
                    match p {
                        Ok(payload) => match payload.command {
                            Advance(a) => {
                                use AdvanceAmount::*;
                                match a {
                                    MasterCycles(n) => {
                                        let goal_cycles = console.total_master_clocks() + n as u64;
                                        while *console.total_master_clocks() < goal_cycles {
                                            // advance!();
                                        }
                                    }
                                    Scanlines(n) => (0..n).for_each(|_| {
                                        let mut hblank = console.ppu().is_in_hblank();
                                        while !(hblank && !console.ppu().is_in_hblank()) {
                                            hblank = console.ppu().is_in_hblank();
                                            // advance!();
                                        }
                                    }),
                                    Instructions(instructions) => {
                                        (0..instructions).for_each(|_| {
                                            // advance!();
                                        });
                                    }
                                    Frames(n) => (0..n).for_each(|_| {
                                        let mut v = console.ppu().is_in_vblank();
                                        while !(!v && console.ppu().is_in_vblank()) {
                                            v = console.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }),
                                    StartVBlank => {
                                        let mut vblank = console.ppu().is_in_vblank();
                                        while !(!vblank && console.ppu().is_in_vblank()) {
                                            vblank = console.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }
                                    EndVBlank => {
                                        let mut vblank = console.ppu().is_in_vblank();
                                        while !(vblank && !console.ppu().is_in_vblank()) {
                                            vblank = console.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }
                                }
                                emu_data_sender
                                    .send(DoneEmuPayload {
                                        console: console.clone(),
                                        profiler,
                                        disassembler,
                                    })
                                    .expect("Unable to send console to main thread");
                            }
                            SetSettings(s) => {
                                // Update settings
                                settings = s;
                            }
                            UpdateInputPorts(input_ports) => {
                                *console.input_ports_mut() = input_ports;
                            }
                            LoadRom(bytes) => {
                                console = Console::with_cartridge(&bytes);
                                disassembler.add_current_instruction(&console);
                                emu_data_sender
                                    .send(DoneEmuPayload {
                                        console: console.clone(),
                                        disassembler,
                                        profiler,
                                    })
                                    .expect("Unable to send console to main thread");
                            }
                            LoadSavestate(c) => {
                                console = c;
                            }
                            Reset => {
                                console.reset();
                            }
                        },
                        Err(_) => {}
                    }
                    // Calculate delta time
                    let now = Instant::now();
                    let dt = now - last_time;
                    last_time = now;
                    // Advance emulator
                    if !settings.is_paused {
                        let initial_master_cycles = console.total_master_clocks().clone();
                        while ((console.total_master_clocks() - initial_master_cycles) as f64)
                            < dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64
                        {
                            advance!();
                        }
                        // Update audio
                        let samples = console.apu_mut().sample_queue();
                        let (a, b) = samples.as_slices();
                        audio.push_samples(a, settings.volume);
                        audio.push_samples(b, settings.volume);
                    }
                    // Sleep
                    thread::sleep(SLEEP_TIME);
                }
            })
            .expect("Unable to spawn thread");

        let console = Console::with_cartridge(include_bytes!("../roms/HelloWorld.sfc"));
        let disassembler = Disassembler::new(&console);
        Engine {
            console,
            sender,
            stream_receiver,
            emu_data_receiver,
            disassembler,
            profiler: Profiler::new(),
            prev_frame_data: [[0; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
        }
    }

    pub fn update(&mut self, command: Command) {
        self.sender
            .send(UpdateEmuPayload {
                command,
                settings: AdvanceSettings::default(),
            })
            .expect("Unable to send data to thread");
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::LoadRom(bytes.to_vec()),
                AdvanceSettings::default(),
            ))
            .expect("Unable to send data to thread");
        // Todo: Tidy this up
        self.disassembler = Disassembler::new(&Console::with_cartridge(bytes));
    }
    pub fn load_savestate(&mut self, bytes: &[u8]) {
        let mut console: Console = serde_brief::from_slice(bytes).unwrap();
        console.ppu_mut().reset_vram_cache();
        self.sender
            .send(UpdateEmuPayload::new(
                Command::LoadSavestate(console),
                AdvanceSettings::default(),
            ))
            .expect("Unable to send data to thread")
    }

    pub fn reset(&mut self) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Reset,
                AdvanceSettings::default(),
            ))
            .expect("Unable to send payload");
    }

    pub fn on_frame(&mut self) {
        // Update console
        match self.emu_data_receiver.try_recv() {
            Ok(mut d) => {
                self.console = d.console;
                self.disassembler.consume(&mut d.disassembler);
                self.profiler.consume(&mut d.profiler);
            }
            Err(_) => {}
        }
        // Update screen data
        match self.stream_receiver.try_recv() {
            Ok(StreamPayload { screen_data: f }) => {
                self.prev_frame_data = core::array::from_fn(|i| [f[i][0], f[i][1], f[i][2]]);
            }
            Err(_) => {}
        }
    }

    pub fn console(&self) -> &Console {
        &self.console
    }
}
