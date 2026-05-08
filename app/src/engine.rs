use derive_new::new;
use log::*;
use slint::SharedString;
use std::{
    collections::VecDeque,
    fmt::Display,
    ops::Deref,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self},
    time::{Duration, Instant},
};
use super_yane::{Console, Cpu, InputPort, MASTER_CLOCK_SPEED_HZ, Ppu, ppu::SCREEN_RESOLUTION};
use wdc65816::Processor;

const SLEEP_TIME: Duration = Duration::from_millis(5);

use crate::{
    ConsoleData, CpuData, PpuData, Settings, apu_snapshot::ApuSnapshot, audio::Audio,
    cpu_snapshot::CpuSnapshot, disassembler::Disassembler, profiler::Profiler,
};

/// Shorthand for converting a byte to a 2-digit hex number
fn h8(value: u8) -> SharedString {
    format!("{:02X}", value).into()
}
/// Shorthand for converting a u16 into a 4-digit hex number
fn h16(value: impl Into<u16>) -> SharedString {
    format!("{:04X}", value.into()).into()
}
/// Shorthand for converting a bool to a 1 or 0 shared string
fn b(value: bool) -> SharedString {
    format!("{}", u8::from(value)).into()
}
impl Into<CpuData> for Processor {
    fn into(self) -> CpuData {
        let Processor {
            a,
            b: b_reg,
            xl,
            xh,
            yl,
            yh,
            pbr,
            pc,
            dbr,
            dl,
            dh,
            s,
            p,
            ..
        } = self;
        CpuData {
            pbr: h8(pbr),
            pc: h16(pc),
            a: h8(a),
            b: h8(b_reg),
            c: h16(self.c()),
            x: h16(self.x()),
            xl: h8(xl),
            xh: h8(xh),
            y: h16(self.y()),
            yl: h8(yl),
            yh: h8(yh),
            sp: h16(s),
            dbr: h8(dbr),
            d: h16(self.dr()),
            dl: h8(dl),
            dh: h8(dh),
            p: h8(p.to_byte(true)),
            p_z: b(p.z),
            p_v: b(p.v),
            p_n: b(p.n),
            p_c: b(p.c),
            p_d: b(p.d),
            p_i: b(p.i),
            p_m: b(p.m),
            p_e: b(p.e),
            p_xb: b(p.xb),
        }
    }
}

impl Into<PpuData> for &Ppu {
    fn into(self) -> PpuData {
        let Ppu {
            vblank,
            forced_blanking,
            brightness,
            bg_mode,
            bg3_prio,
            mosaic_size,
            vram_addr,
            vram_increment_mode,
            vram_increment_amount,
            cgram_addr,
            ..
        } = *self;
        PpuData {
            vblank: b(vblank),
            forced_blanking: b(forced_blanking),
            brightness: h8(brightness),
            bg_mode: h8(bg_mode as u8),
            bg3_prio: b(bg3_prio),
            mosaic_size: h8(mosaic_size as u8),
            vram_addr: h16(vram_addr as u16),
            vram_inc_mode: vram_increment_mode.to_string().into(),
            vram_inc_amt: h16(vram_increment_amount as u16),
            cgram_addr: h16(cgram_addr as u16),
        }
    }
}

impl Into<ConsoleData> for &Console {
    fn into(self) -> ConsoleData {
        ConsoleData {
            cpu: (*self.cpu()).into(),
            ppu: self.ppu().into(),
        }
    }
}

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
    pub disassembler: Disassembler,
    pub profiler: Profiler,
    sender: Sender<UpdateEmuPayload>,
    stream_receiver: Receiver<StreamPayload>,
    /// The RGB data from the previous fully rendered frame
    pub prev_frame_data: [[u8; 3]; SCREEN_RESOLUTION[0] * SCREEN_RESOLUTION[1]],
}
impl Engine {
    pub fn new(
        console: Arc<Mutex<Console>>,
        settings: Arc<Mutex<Settings>>,
        data: Arc<Mutex<ConsoleData>>,
    ) -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (sender, receiver) = mpsc::channel::<UpdateEmuPayload>();
        // Send new frame data every time a new frame is generated
        let (stream_sender, stream_receiver) = mpsc::channel::<StreamPayload>();
        // Initialize audio
        let mut audio = Audio::new();

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                use Command::*;
                // Used to calculate delta time to advance the emulator
                let mut last_time = Instant::now();
                loop {
                    // let mut disassembler = Disassembler::new(&console);
                    // let mut profiler = Profiler::new();
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
                            // disassembler.add_current_instruction(&console);
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
                                            // advance!();
                                        }
                                    }
                                    Scanlines(n) => (0..n).for_each(|_| {
                                        let mut hblank = c.ppu().is_in_hblank();
                                        while !(hblank && !c.ppu().is_in_hblank()) {
                                            hblank = c.ppu().is_in_hblank();
                                            // advance!();
                                        }
                                    }),
                                    Instructions(instructions) => {
                                        (0..instructions).for_each(|_| {
                                            // advance!();
                                        });
                                    }
                                    Frames(n) => (0..n).for_each(|_| {
                                        let mut v = c.ppu().is_in_vblank();
                                        while !(!v && c.ppu().is_in_vblank()) {
                                            v = c.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }),
                                    StartVBlank => {
                                        let mut vblank = c.ppu().is_in_vblank();
                                        while !(!vblank && c.ppu().is_in_vblank()) {
                                            vblank = c.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }
                                    EndVBlank => {
                                        let mut vblank = c.ppu().is_in_vblank();
                                        while !(vblank && !c.ppu().is_in_vblank()) {
                                            vblank = c.ppu().is_in_vblank();
                                            // advance!();
                                        }
                                    }
                                }
                            }
                            UpdateInputPorts(input_ports) => {
                                *c.input_ports_mut() = input_ports;
                            }
                            LoadRom(bytes) => {
                                *c = Console::with_cartridge(&bytes);
                                // disassembler.add_current_instruction(&console);
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
                    // Copy data
                    *data.lock().unwrap() = c.deref().into();
                    // Sleep
                    thread::sleep(SLEEP_TIME);
                }
            })
            .expect("Unable to spawn thread");

        let console = Console::with_cartridge(include_bytes!("../roms/HelloWorld.sfc"));
        let disassembler = Disassembler::new(&console);
        Engine {
            sender,
            stream_receiver,
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
        // Update screen data
        match self.stream_receiver.try_recv() {
            Ok(StreamPayload { screen_data: f }) => {
                self.prev_frame_data = core::array::from_fn(|i| [f[i][0], f[i][1], f[i][2]]);
            }
            Err(_) => {}
        }
    }
}
