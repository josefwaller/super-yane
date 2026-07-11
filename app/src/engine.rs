// use crate::{
//     AppWindow, DisassemblyLine, OamData,
//     utils::{get_oam_data, update_binary_data},
// };
use closure::closure;
use derive_new::new;
use egui::Context;
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
struct Settings {
    is_paused: bool,
    log_apu: bool,
    log_cpu: bool,
    volume: f32,
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
    pub console: Console,
    pub settings: Settings,
    pub cpu_dis: Disassembler<CpuInstruction>,
    pub apu_dis: Disassembler<ApuInstruction>,
    pub ctx: Context,
}

impl Emulation {
    fn advance(&mut self) {
        let c = &mut self.console;
        let s = &mut self.settings;
        let cpu_dis = &mut self.cpu_dis;
        let apu_dis = &mut self.apu_dis;
        let pc = c.pc();
        // let before_master_cycles = *c.total_master_clocks();
        c.step_cpu();
        cpu_dis.add_current_instruction(&c);
        if s.log_cpu && c.pc() != pc {
            let inst = CpuSnapshot::from(&c);
            info!("[CPU] {}", inst);
        }
        while c.apu_is_behind() {
            c.step_apu();
            apu_dis.add_current_instruction(&c);
            if s.log_apu {
                let inst = ApuSnapshot::from(&c);
                info!("[APU] {}", inst);
            }
        }
        // profiler.add_current_state(&console, before_master_cycles);
    }
    pub fn on_command(&mut self, command: Command) {
        use Command::*;
        match command {
            Advance(a) => {
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
            UpdateInputPorts(input_ports) => {
                *self.console.input_ports_mut() = input_ports;
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
        };
    }
}

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    to_emu: Sender<UpdateEmuPayload>,
    pub emulation: Arc<Mutex<Emulation>>,
}

// fn update_ui(emulation: Arc<Mutex<Emulation>>, ui_ptr: Weak<AppWindow>) {
//     // Clone the Arc<Mutex<Console>> instead of the console here
//     ui_ptr
//         .upgrade_in_event_loop(closure!(clone emulation, |ui| {
//             let e = emulation.lock().unwrap();
//             let c = &e.console;
//             let cpu_dis = &e.cpu_dis;
//             let apu_dis = &e.apu_dis;
//             let mut buf: SharedPixelBuffer<Rgb8Pixel> = if ui.get_pixel_data().size().width == 0 {
//                 SharedPixelBuffer::new(256, 224)
//             } else {
//                 ui.get_pixel_data().to_rgb8().unwrap()
//             };
//             buf.make_mut_bytes().copy_from_slice(c.ppu().screen_data_rgb().as_flattened());
//             let pc = c.pc();
//             let pc = c.cartridge().transform_address(pc);
//             let cpu_dis_lines = cpu_dis.slint_instructions(pc, 16, 16);
//             let apu_dis_lines = apu_dis.slint_instructions(c.apu().core.pc as usize, 16, 16);
//             update_binary_data(
//                 &c,
//                 ui.get_binary_data_offset() as usize,
//                 ui.get_binary_src(),
//                 ui.get_bpp(),
//                 ui.get_palette_index() as usize,
//                 &ui
//             );
//             ui.get_binary_data().iter().for_each(|row| row.set_row_data(0, 4));
//             ui.set_console_data(c.into());
//             ui.set_pixel_data(Image::from_rgb8(buf));

//             if ui.get_cpu_disassembly_lines().row_count() == 0 {
//                 ui.set_cpu_disassembly_lines(ModelRc::new(VecModel::from(cpu_dis_lines)));
//                 ui.set_apu_disassembly_lines(ModelRc::new(VecModel::from(apu_dis_lines)));
//             } else {
//                 cpu_dis_lines.into_iter().enumerate().for_each(|(index, line)|
//                     ui.get_cpu_disassembly_lines().set_row_data(index, line));
//                 apu_dis_lines.into_iter().enumerate().for_each(|(index, line)|
//                     ui.get_apu_disassembly_lines().set_row_data(index, line));
//             }

//             ui.set_backgrounds(ModelRc::from(
//                 Rc::from(VecModel::from_iter(
//                     c.ppu().backgrounds.iter().map(|b| b.into())
//                 ))
//             ));
//             // Set up OAM data
//             if ui.get_oam_data().row_count() < c.ppu().oam_sprites.len() {
//                 // Initialize OAM ModelRc
//                 ui.set_oam_data(
//                     ModelRc::from(Rc::from(VecModel::from_iter(c.ppu().oam_sprites.iter().map(
//                         |o| get_oam_data(o, c.ppu())
//                     )))));
//             } else {
//                 // Update in place
//                 c.ppu().oam_sprites.iter().enumerate().for_each(
//                     |(i, o)| {
//                         ui.get_oam_data().set_row_data(i, get_oam_data(o, c.ppu()))
//                     },
//                 );
//             }
//         }))
//         .unwrap();
// }

impl Engine {
    pub fn new(
        console: Console, //     , ui_ptr: Weak<AppWindow>
        ctx: Context,
    ) -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (to_emu, from_main) = mpsc::channel::<UpdateEmuPayload>();
        // Initialize audio
        let mut audio = Audio::new();
        let emulation = Arc::new(Mutex::new(Emulation {
            console,
            settings: Settings {
                volume: 20.0,
                is_paused: false,
                log_apu: false,
                log_cpu: false,
            },
            cpu_dis: Disassembler::<CpuInstruction>::new(),
            apu_dis: Disassembler::<ApuInstruction>::new(),
            ctx,
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
                        let s = e.settings.clone();
                        // Advance emulator
                        if !s.is_paused {
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
                            audio.push_samples(a, s.volume);
                            audio.push_samples(b, s.volume);

                        }
                        // Repaint
                        e.ctx.request_repaint();
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

    pub fn update_settings(&mut self, settings: Settings) {
        self.emulation.lock().unwrap().settings = settings;
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
