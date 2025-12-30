use derive_new::new;
use log::*;
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self},
    time::Duration,
};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ};

const DEFAULT_CARTRIDGE: &[u8] = include_bytes!("../roms/HelloWorld.sfc");

use crate::instruction_snapshot::InstructionSnapshot;

/// Misc settings for advancing the emulator
#[derive(Default, Clone)]
pub struct AdvanceSettings {
    /// Whether to log the CPU state after every instruction
    pub log_cpu: bool,
}

/// Command send to the emulation thread
pub enum Command {
    MasterCycles(u64),
    Instructions(u32),
    ToVBlank,
    LoadRom(Vec<u8>),
    LoadSavestate(Console),
    Reset,
}
/// The payload send to the emulation thread telling it to update the emulator
#[derive(new)]
pub struct UpdateEmuPayload {
    /// How much to advance the emulator by
    advance_by: Command,
    /// The current input
    input: [InputPort; 2],
    /// The settings
    settings: AdvanceSettings,
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
    sender: Sender<UpdateEmuPayload>,
    console_receiver: Receiver<Console>,
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
        // Send the console back to the main thread after emulating
        let (console_sender, console_receiver) = mpsc::channel::<Console>();
        // Send new frame data every time a new frame is generated
        let (stream_sender, stream_receiver) = mpsc::channel::<StreamPayload>();

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                use Command::*;
                let mut console = Console::with_cartridge(DEFAULT_CARTRIDGE);
                loop {
                    let payload = receiver.recv().unwrap();
                    console.input_ports_mut()[0] = payload.input[0];
                    match payload.advance_by {
                        MasterCycles(n) => {
                            let goal_cycles = console.total_master_clocks() + n;
                            while *console.total_master_clocks() < goal_cycles {
                                let vblank = console.ppu().is_in_vblank();
                                if payload.settings.log_cpu {
                                    let inst = InstructionSnapshot::from(&console);
                                    debug!("{}", inst);
                                }
                                console.advance_instructions(1);
                                if vblank && !console.ppu().is_in_vblank() {
                                    stream_sender
                                        .send(StreamPayload::new(
                                            console.ppu().screen_data_rgb(),
                                            console.apu_mut().sample_queue(),
                                        ))
                                        .expect("Unable to send frame data");
                                }
                            }
                            console_sender
                                .send(console.clone())
                                .expect("Unable to send console to main thread");
                        }
                        Instructions(instructions) => {
                            console.advance_instructions(instructions);
                            console_sender
                                .send(console.clone())
                                .expect("Unable to send console back to main thread");
                        }
                        ToVBlank => {
                            let mut vblank = console.ppu().is_in_vblank();
                            while !vblank && console.ppu().is_in_vblank() {
                                vblank = console.ppu().is_in_vblank();
                                console.advance_instructions(1);
                            }
                            debug!("Done advancing");
                            stream_sender
                                .send(StreamPayload::new(
                                    console.ppu().screen_data_rgb(),
                                    console.apu_mut().sample_queue(),
                                ))
                                .expect("Unable to send frame data");

                            console_sender
                                .send(console.clone())
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
            console_receiver,
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
                Command::MasterCycles(cycles),
                self.input_ports.clone(),
                settings,
            ))
            .expect("Unable to send to console thread");
    }
    pub fn advance_frames(&mut self, num_frames: u32, settings: AdvanceSettings) {
        (0..num_frames).for_each(|_| {
            self.sender
                .send(UpdateEmuPayload::new(
                    Command::ToVBlank,
                    self.input_ports.clone(),
                    settings.clone(),
                ))
                .expect("Unable to send to console thread")
        });
    }
    /// Advance the console by a number of instructions
    pub fn advance_instructions(&mut self, instructions: u32, settings: AdvanceSettings) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Instructions(instructions),
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
        match self.console_receiver.try_recv() {
            Ok(c) => self.console = c,
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
