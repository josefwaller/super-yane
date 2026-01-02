use derive_new::new;
use log::*;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self},
    time::Duration,
};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ};

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");

use crate::{
    disassembler::{self, Disassembler},
    instruction_snapshot::InstructionSnapshot,
};

/// Misc settings for advancing the emulator
#[derive(Default, Clone)]
pub struct AdvanceSettings {
    /// Whether to log the CPU state after every instruction
    pub log_cpu: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceAmount {
    MasterCycles(u64),
    Instructions(u32),
    Frames(usize),
    StartVBlank,
    EndVBlank,
}
/// Command send to the emulation thread
pub enum Command {
    Advance(AdvanceAmount),
    LoadRom(Vec<u8>),
    LoadSavestate(Console),
    Reset,
}
/// The payload send to the emulation thread telling it to update the emulator
#[derive(new)]
pub struct UpdateEmuPayload {
    /// How much to advance the emulator by
    command: Command,
    /// The current input
    input: [InputPort; 2],
    /// The settings
    settings: AdvanceSettings,
}

/// The Payload to send data back to the main thread after finishing emulation
#[derive(new)]
pub struct DoneEmuPayload {
    console: Console,
    disassembler: Disassembler,
}

/// The payload sent every frame from the emulator thread containing output information
#[derive(new)]
pub struct StreamPayload {
    screen_data: [[u8; 3]; 256 * 240],
    samples: VecDeque<f32>,
}

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    console: Console,
    pub disassembler: Disassembler,
    sender: Sender<UpdateEmuPayload>,
    emu_data_receiver: Receiver<DoneEmuPayload>,
    stream_receiver: Receiver<StreamPayload>,
    pub input_ports: [InputPort; 2],
    /// The RGB data from the previous fully rendered frame
    pub prev_frame_data: [[u8; 4]; 256 * 240],
    /// Audio sample queue
    samples: VecDeque<f32>,
}
impl Engine {
    pub fn new() -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (sender, receiver) = mpsc::channel::<UpdateEmuPayload>();
        // Send data back to the main thread after emulating
        let (emu_data_sender, emu_data_receiver) = mpsc::channel::<DoneEmuPayload>();
        // Send new frame data every time a new frame is generated
        let (stream_sender, stream_receiver) = mpsc::channel::<StreamPayload>();

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                use Command::*;
                let mut console = Console::with_cartridge(DEFAULT_CARTRIDGE);
                loop {
                    let payload = receiver.recv().unwrap();
                    let mut disassembler = Disassembler::new();
                    console.input_ports_mut()[0] = payload.input[0];
                    /// Advance by 1 instruction
                    macro_rules! advance {
                        () => {
                            let vblank = console.ppu().is_in_vblank();
                            console.advance_instructions(1);
                            if payload.settings.log_cpu {
                                let inst = InstructionSnapshot::from(&console);
                                info!("{}", inst);
                            }
                            disassembler.add_current_instruction(&console);
                            if vblank && !console.ppu().is_in_vblank() {
                                stream_sender
                                    .send(StreamPayload::new(
                                        console.ppu().screen_data_rgb(),
                                        console.apu_mut().sample_queue(),
                                    ))
                                    .expect("Unable to send frame data");
                            }
                        };
                    }
                    match payload.command {
                        Advance(a) => {
                            use AdvanceAmount::*;
                            match a {
                                MasterCycles(n) => {
                                    let goal_cycles = console.total_master_clocks() + n;
                                    while *console.total_master_clocks() < goal_cycles {
                                        advance!();
                                    }
                                }
                                Instructions(instructions) => {
                                    (0..instructions).for_each(|_| {
                                        advance!();
                                    });
                                }
                                StartVBlank => {
                                    let mut vblank = console.ppu().is_in_vblank();
                                    while !vblank && console.ppu().is_in_vblank() {
                                        vblank = console.ppu().is_in_vblank();
                                        advance!();
                                    }
                                    stream_sender
                                        .send(StreamPayload::new(
                                            console.ppu().screen_data_rgb(),
                                            console.apu_mut().sample_queue(),
                                        ))
                                        .expect("Unable to send frame data");
                                }
                                _ => todo!(),
                            }
                            emu_data_sender
                                .send(DoneEmuPayload {
                                    console: console.clone(),
                                    disassembler,
                                })
                                .expect("Unable to send console to main thread");
                        }
                        LoadRom(bytes) => {
                            console = Console::with_cartridge(&bytes);
                        }
                        LoadSavestate(c) => {
                            console = c;
                        }
                        Reset => {
                            console.reset();
                        }
                    }
                }
            })
            .expect("Unable to spawn thread");

        Engine {
            console: Console::with_cartridge(include_bytes!("../roms/HelloWorld.sfc")),
            sender,
            stream_receiver,
            emu_data_receiver,
            disassembler: Disassembler::new(),
            input_ports: [InputPort::default_standard_controller(); 2],
            prev_frame_data: [[0; 4]; 256 * 240],
            samples: VecDeque::new(),
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::LoadRom(bytes.to_vec()),
                self.input_ports,
                AdvanceSettings::default(),
            ))
            .expect("Unable to send data to thread");
        self.disassembler = Disassembler::new();
    }
    pub fn load_savestate(&mut self, bytes: &[u8]) {
        let mut console: Console = serde_brief::from_slice(bytes).unwrap();
        console.ppu_mut().reset_vram_cache();
        self.sender
            .send(UpdateEmuPayload::new(
                Command::LoadSavestate(console),
                self.input_ports,
                AdvanceSettings::default(),
            ))
            .expect("Unable to send data to thread")
    }

    /// Advance the console by a given duration
    pub fn advance_dt(&mut self, dt: Duration, settings: AdvanceSettings) {
        let cycles =
            (dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64).floor() as u64;
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Advance(AdvanceAmount::MasterCycles(cycles)),
                self.input_ports.clone(),
                settings,
            ))
            .expect("Unable to send to console thread");
    }
    /// Advance the console by a number of instructions
    pub fn advance_amount(&mut self, amount: AdvanceAmount, settings: AdvanceSettings) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Advance(amount),
                self.input_ports.clone(),
                settings,
            ))
            .expect("Unable to send to console thread");
    }

    pub fn reset(&mut self) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Reset,
                self.input_ports.clone(),
                AdvanceSettings::default(),
            ))
            .expect("Unable to send payload");
    }

    pub fn on_frame(&mut self) {
        // Update console
        match self.emu_data_receiver.try_recv() {
            Ok(mut d) => {
                self.console = d.console;
                self.disassembler.merge(&mut d.disassembler);
            }
            Err(_) => {}
        }
        // Update screen data
        match self.stream_receiver.try_recv() {
            Ok(StreamPayload {
                screen_data: f,
                samples,
            }) => {
                self.prev_frame_data = core::array::from_fn(|i| [f[i][0], f[i][1], f[i][2], 0xFF]);
                self.samples.extend(samples);
            }
            Err(_) => {}
        }
    }

    /// Retrieve the audio samples generated so far from the internal buffer
    /// and clear the buffer
    pub fn swap_samples(&mut self) -> VecDeque<f32> {
        let mut s = VecDeque::new();
        std::mem::swap(&mut self.samples, &mut s);
        s
    }

    pub fn console(&self) -> &Console {
        &self.console
    }
}
