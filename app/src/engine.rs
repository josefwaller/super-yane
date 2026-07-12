// use crate::{
//     AppWindow, DisassemblyLine, OamData,
//     utils::{get_oam_data, update_binary_data},
// };
use closure::closure;
use derive_new::new;
use egui::Context as UiContext;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread::{self},
    time::{Duration, Instant},
};
use super_yane::{Console, MASTER_CLOCK_SPEED_HZ};

const SLEEP_TIME: Duration = Duration::from_millis(5);

use crate::{audio::Audio, emulation::Emulation};

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
        // Initialize emulation thread
        let emulation = Arc::new(Mutex::new(Emulation::new(console, ui_ctx)));

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(closure!(clone emulation,
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
                                    e.post_advance();
                                }
                            }
                            // Update audio
                            let samples = e.console.apu_mut().sample_queue();
                            let (a, b) = samples.as_slices();
                            audio.push_samples(a, e.volume);
                            audio.push_samples(b, e.volume);

                        }
                    }
                        // Sleep
                        thread::sleep(SLEEP_TIME);
                    }
            }))
            .expect("Unable to spawn thread");

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
