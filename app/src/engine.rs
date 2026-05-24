use crate::{
    AppWindow, DisassemblyLine,
    utils::{bytes_to_rgb, get_binary_data},
};
use closure::closure;
use derive_new::new;
use log::*;
use slint::{Image, ModelRc, SharedPixelBuffer, SharedString, VecModel};
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
use super_yane::{
    Console, Cpu, InputPort, MASTER_CLOCK_SPEED_HZ, Ppu, ppu::SCREEN_RESOLUTION,
    utils::color_to_rgb_bytes,
};
use wdc65816::Processor;

const SLEEP_TIME: Duration = Duration::from_millis(5);

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");

use crate::{
    ConsoleData, CpuData, PpuData, Settings, apu_snapshot::ApuSnapshot, audio::Audio,
    cpu_snapshot::CpuSnapshot, disassembler::Disassembler, profiler::Profiler,
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
    // The console that the engine is running
    pub console: Arc<Mutex<Console>>,
    pub settings: Arc<Mutex<Settings>>,
    pub disassembler: Arc<Mutex<Disassembler>>,
    pub profiler: Profiler,
    sender: Sender<UpdateEmuPayload>,
    stream_receiver: Receiver<StreamPayload>,
    /// The RGB data from the previous fully rendered frame
    pub prev_frame_data: [[u8; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
}
impl Engine {
    pub fn new() -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (sender, receiver) = mpsc::channel::<UpdateEmuPayload>();
        // Send new frame data every time a new frame is generated
        let (stream_sender, stream_receiver) = mpsc::channel::<StreamPayload>();
        // Initialize audio
        let mut audio = Audio::new();
        // Initialize disassembler
        let disassembler = Arc::new(Mutex::new(Disassembler::new()));
        // Initialize console
        let console = Arc::new(Mutex::new(Console::with_cartridge(DEFAULT_CARTRIDGE)));
        // Initialize settings
        let settings = Arc::new(Mutex::new(Settings {
            volume: 20.0,
            is_paused: false,
        }));
        disassembler
            .lock()
            .unwrap()
            .add_native_vectors(console.lock().unwrap().deref());

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(closure!(clone disassembler, clone console, clone settings, || {
                use Command::*;
                // Used to calculate delta time to advance the emulator
                let mut last_time = Instant::now();
                loop {
                    // let mut profiler = Profiler::new();
                    let mut d = Disassembler::new();
                    // New scope so that c lives as short as possible
                    {
                        // Get a lock on the console
                        let mut c = console.lock().unwrap();
                        /// Advance by 1 instruction
                        macro_rules! advance {
                            () => {
                                let vblank = c.ppu().is_in_vblank();
                                let before_master_cycles = *c.total_master_clocks();
                                c.step_cpu();
                                // if payload.settings.log_cpu {
                                //     let inst = CpuSnapshot::from(&console);
                                //     info!("{}", inst);
                                // }
                                while c.apu_is_behind() {
                                    c.step_apu();
                                    // if payload.settings.log_apu {
                                    //     let inst = ApuSnapshot::from(&console);
                                    //     info!("{}", inst);
                                    // }
                                }
                                d.add_current_instruction(&c);
                                // profiler.add_current_state(&console, before_master_cycles);

                                if vblank && !c.ppu().is_in_vblank() {
                                    stream_sender
                                        .send(StreamPayload::new(c.ppu().screen_data_rgb()))
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
                                            let goal_cycles = c.total_master_clocks() + n as u64;
                                            while *c.total_master_clocks() < goal_cycles {
                                                advance!();
                                            }
                                        }
                                        Scanlines(n) => (0..n).for_each(|_| {
                                            let mut hblank = c.ppu().is_in_hblank();
                                            while !(hblank && !c.ppu().is_in_hblank()) {
                                                hblank = c.ppu().is_in_hblank();
                                                advance!();
                                            }
                                        }),
                                        Instructions(instructions) => {
                                            (0..instructions).for_each(|_| {
                                                advance!();
                                            });
                                        }
                                        Frames(n) => (0..n).for_each(|_| {
                                            let mut v = c.ppu().is_in_vblank();
                                            while !(!v && c.ppu().is_in_vblank()) {
                                                v = c.ppu().is_in_vblank();
                                                advance!();
                                            }
                                        }),
                                        StartVBlank => {
                                            let mut vblank = c.ppu().is_in_vblank();
                                            while !(!vblank && c.ppu().is_in_vblank()) {
                                                vblank = c.ppu().is_in_vblank();
                                                advance!();
                                            }
                                        }
                                        EndVBlank => {
                                            let mut vblank = c.ppu().is_in_vblank();
                                            while !(vblank && !c.ppu().is_in_vblank()) {
                                                vblank = c.ppu().is_in_vblank();
                                                advance!();
                                            }
                                        }
                                    }
                                }
                                UpdateInputPorts(input_ports) => {
                                    *c.input_ports_mut() = input_ports;
                                }
                                LoadRom(bytes) => {
                                    *c = Console::with_cartridge(&bytes);
                                }
                                LoadSavestate(state) => {
                                    *c = state;
                                }
                                Reset => {
                                    c.reset();
                                }
                            },
                            Err(_) => {}
                        }
                        // Calculate delta time
                        let now = Instant::now();
                        let dt = now - last_time;
                        last_time = now;
                        // Clone settings
                        let s = settings.lock().unwrap().clone();
                        // Advance emulator
                        if !s.is_paused {
                            let initial_master_cycles = c.total_master_clocks().clone();
                            while ((c.total_master_clocks() - initial_master_cycles) as f64)
                                < dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64
                            {
                                advance!();
                            }
                            // Update audio
                            let samples = c.apu_mut().sample_queue();
                            let (a, b) = samples.as_slices();
                            audio.push_samples(a, s.volume);
                            audio.push_samples(b, s.volume);
                        }
                    }
                    // Merge disassembler
                    disassembler.lock().unwrap().consume(&mut d);
                    // Sleep
                    thread::sleep(SLEEP_TIME);
                }
            }))
            .expect("Unable to spawn thread");

        Engine {
            sender,
            stream_receiver,
            console,
            settings,
            disassembler,
            profiler: Profiler::new(),
            prev_frame_data: [[0; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
        }
    }

    pub fn console<'a>(&'a self) -> MutexGuard<'a, Console> {
        self.console.lock().expect("Unable to get lock on console")
    }
    pub fn console_data(&self) -> ConsoleData {
        // Copy console data
        let c = self.console();

        ConsoleData {
            cpu: c.cpu().into(),
            ppu: c.ppu().into(),
        }
    }
    /// Update all the info for the binary data viewer.
    /// Data, column headers, row headers, offset, etc.
    pub fn refresh_binary_data(&self, ui: AppWindow) {
        let c = self.console();
        let (data, img_data, len) = get_binary_data(
            &*c,
            ui.get_binary_data_offset() as usize,
            ui.get_binary_src(),
            ui.get_bpp(),
            ui.get_palette_index() as usize,
        );
        ui.set_binary_data(data);
        ui.set_binary_image(Image::from_rgb8(img_data));
        ui.set_binary_data_len(len as i32);
    }
    pub fn update_settings(&mut self, settings: Settings) {
        *self.settings.lock().unwrap() = settings;
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
        // Update screen data
        match self.stream_receiver.try_recv() {
            Ok(StreamPayload { screen_data: f }) => {
                self.prev_frame_data = core::array::from_fn(|i| [f[i][0], f[i][1], f[i][2]]);
            }
            Err(_) => {}
        }
    }
    pub fn get_savestate(&self) -> Vec<u8> {
        let c = self.console().clone();
        serde_brief::to_vec::<Console>(&c).expect("Unable to serialize console")
    }
    pub fn load_savestate(&mut self, state: &[u8]) -> Result<(), serde_brief::Error> {
        let mut c: Console = serde_brief::from_slice(state)?;
        c.ppu_mut().reset_vram_cache();
        *self.console.lock().unwrap() = c;
        Ok(())
    }

    pub fn disassembly_lines(&self, pc: usize) -> Vec<DisassemblyLine> {
        self.disassembler
            .lock()
            .unwrap()
            .instructions()
            .iter()
            .filter(|(i, _)| **i >= pc)
            .take(32)
            .map(|(_, inst)| {
                let d = inst.data();
                DisassemblyLine {
                    instruction: d.name.into(),
                    arguments: inst.operands(&BTreeMap::new()).into(),
                }
            })
            .collect()
    }
}
