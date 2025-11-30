use derive_new::new;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self},
    time::Duration,
};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ};

/// Command send to the emulation thread
pub enum Command {
    MasterCycles(u64),
    LoadRom(Vec<u8>),
    Reset,
}
/// The payload send to the emulation thread telling it to update the emulator
#[derive(new)]
pub struct UpdateEmuPayload {
    /// How much to advance the emulator by
    advance_by: Command,
    /// The current input
    input: [InputPort; 2],
}

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    console: Console,
    sender: Sender<UpdateEmuPayload>,
    console_receiver: Receiver<Console>,
    frame_receiver: Receiver<[[u8; 3]; 256 * 240]>,
    pub input_ports: [InputPort; 2],
    /// The RGB data from the previous fully rendered frame
    pub prev_frame_data: [[u8; 4]; 256 * 240],
}
impl Engine {
    pub fn new() -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (sender, receiver) = mpsc::channel::<UpdateEmuPayload>();
        // Send the console back to the main thread after emulating
        let (console_sender, console_receiver) = mpsc::channel::<Console>();
        // Send new frame data every time a new frame is generated
        let (frame_sender, frame_receiver) = mpsc::channel::<[[u8; 3]; 256 * 240]>();

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                use Command::*;
                let mut console = Console::with_cartridge(include_bytes!("../roms/HelloWorld.sfc"));
                loop {
                    let payload = receiver.recv().unwrap();
                    console.input_ports_mut()[0] = payload.input[0];
                    match payload.advance_by {
                        MasterCycles(n) => {
                            let goal_cycles = console.total_master_clocks() + n;
                            while *console.total_master_clocks() < goal_cycles {
                                let vblank = console.in_vblank();
                                console.advance_instructions(1);
                                if !vblank && console.in_vblank() {
                                    frame_sender
                                        .send(console.ppu().screen_data_rgb())
                                        .expect("Unable to send frame data");
                                }
                            }
                            console_sender
                                .send(console.clone())
                                .expect("Unable to send console to main thread");
                            // Todo tidy: Just clear the queue
                            let _ = console.apu_mut().sample_queue();
                        }
                        LoadRom(bytes) => {
                            console = Console::with_cartridge(&bytes);
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
            frame_receiver,
            console_receiver,
            input_ports: [InputPort::default_standard_controller(); 2],
            prev_frame_data: [[0; 4]; 256 * 240],
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::LoadRom(bytes.to_vec()),
                self.input_ports,
            ))
            .expect("Unable to send data to thread");
    }

    /// Advance the console by a given duration
    pub fn advance_dt(&mut self, dt: Duration) {
        let cycles =
            (dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64).floor() as u64;
        self.sender
            .send(UpdateEmuPayload::new(
                Command::MasterCycles(cycles),
                self.input_ports.clone(),
            ))
            .expect("Unable to send to console thread");
    }

    pub fn reset(&mut self) {
        self.sender
            .send(UpdateEmuPayload::new(
                Command::Reset,
                self.input_ports.clone(),
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
        match self.frame_receiver.try_recv() {
            Ok(f) => {
                self.prev_frame_data = core::array::from_fn(|i| [f[i][0], f[i][1], f[i][2], 0xFF])
            }
            Err(_) => {}
        }
    }

    pub fn console(&self) -> &Console {
        &self.console
    }
}
