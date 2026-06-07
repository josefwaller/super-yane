use crate::{
    AppWindow, DisassemblyLine,
    utils::{bytes_to_rgb, get_binary_data},
};
use closure::closure;
use derive_new::new;
use log::*;
use slint::{Image, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, VecModel, Weak};
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
    ConsoleData, Settings,
    apu_snapshot::ApuSnapshot,
    audio::Audio,
    cpu_snapshot::CpuSnapshot,
    disassembler::{ApuInstruction, CpuInstruction, Disassembler, Instruction},
    profiler::Profiler,
};

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

/// The underlying engine of the emulator application
/// Runs the application on a separate thread and sends data back and forth
pub struct Engine {
    to_emu: Sender<UpdateEmuPayload>,
    console: Arc<Mutex<Console>>,
    settings: Arc<Mutex<Settings>>,
    pub cpu_dis: Arc<Mutex<Disassembler<CpuInstruction>>>,
    pub apu_dis: Arc<Mutex<Disassembler<ApuInstruction>>>,
}

impl Engine {
    pub fn new(console: Console, ui_ptr: Weak<AppWindow>) -> Engine {
        // Send data to the emulation thread telling it to update the emulator
        let (to_emu, from_main) = mpsc::channel::<UpdateEmuPayload>();
        // Initialize audio
        let mut audio = Audio::new();
        // Create console
        let console = Arc::new(Mutex::new(console));
        // Create initial settings
        let settings = Arc::new(Mutex::new(Settings {
            volume: 20.0,
            is_paused: false,
            log_apu: false,
            log_cpu: false,
        }));

        // Create disassmblers
        let cpu_dis = Arc::new(Mutex::new(Disassembler::<CpuInstruction>::new()));
        let apu_dis = Arc::new(Mutex::new(Disassembler::<ApuInstruction>::new()));

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(closure!(clone console, clone settings, clone cpu_dis, clone apu_dis, || {
                {
                    let d = &mut cpu_dis.lock().unwrap();
                    // Add initial vectors and instruction
                    d.add_native_vectors(&console.lock().unwrap());
                    d.add_current_instruction(&console.lock().unwrap());
                }
                apu_dis.lock().unwrap().add_current_instruction(&console.lock().unwrap());

                // Used to calculate delta time to advance the emulator
                let mut last_time = Instant::now();
                loop {
                    /// Advance by 1 instruction
                    macro_rules! advance {
                        ($console: ident, $settings: ident) => {
                            // let before_master_cycles = *c.total_master_clocks();
                            $console.step_cpu();
                            cpu_dis.lock().unwrap().add_current_instruction(&$console);
                            if $settings.log_cpu {
                                let inst = CpuSnapshot::from(&$console);
                                info!("[CPU] {}", inst);
                            }
                            while $console.apu_is_behind() {
                                $console.step_apu();
                                apu_dis.lock().unwrap().add_current_instruction(&$console);
                                if $settings.log_apu {
                                    let inst = ApuSnapshot::from(&$console);
                                    info!("[APU] {}", inst);
                                }
                            }
                            // profiler.add_current_state(&console, before_master_cycles);
                        };
                    }
                       // Send data back to the main thread for slint to display
                        macro_rules! update_ui {
                            ($c: ident, $s: ident) => {
                                let buf: SharedPixelBuffer<Rgb8Pixel> =
                                    SharedPixelBuffer::clone_from_slice(
                                        $c.ppu().screen_data_rgb().as_flattened(),
                                        256,
                                        224,
                                    );
                                let pc = $c.pc();
                                let pc = $c.cartridge().transform_address(pc);
                                let cpu_dis_lines = cpu_dis.lock().unwrap().slint_instructions(pc, 16, 16);
                                let apu_dis_lines = cpu_dis.lock().unwrap().slint_instructions($c.apu().core.pc as usize, 16, 16);
                                // Clone the Arc<Mutex<Console>> instead of the console here
                                ui_ptr
                                    .upgrade_in_event_loop(closure!(clone console, |ui| {
                                        let c = console.lock().expect("Unable to get a lock on console");
                                        let (data, img_data, len) =
                                            get_binary_data(
                                                &c,
                                                ui.get_binary_data_offset() as usize,
                                                ui.get_binary_src(),
                                                ui.get_bpp(),
                                                ui.get_palette_index() as usize,
                                            );
                                        ui.set_console_data(c.deref().into());
                                        ui.set_pixel_data(Image::from_rgb8(buf));
                                        ui.set_binary_data(data);
                                        ui.set_binary_image(Image::from_rgb8(img_data));
                                        ui.set_binary_data_len(len as i32);

                                        ui.set_cpu_disassembly_lines(ModelRc::new(VecModel::from(cpu_dis_lines)));
                                        ui.set_apu_disassembly_lines(ModelRc::new(VecModel::from(apu_dis_lines)));

                                        ui.set_backgrounds(ModelRc::from(
                                            Rc::from(VecModel::from_iter(
                                                c.ppu().backgrounds.iter().map(|b| b.into())
                                            ))
                                        ));
                                    }))
                                    .unwrap();
                            };
                        }
                        let p = from_main.try_recv();
                        use Command::*;
                        match p {
                            Ok(payload) => {
                                // Get a lock on the console
                                let mut c = console
                                    .lock()
                                    .expect("Unable to get a lock on console in console thread");
                                let s = settings.lock().unwrap();

                                match payload.command {
                                Advance(a) => {
                                    use AdvanceAmount::*;
                                    match a {
                                        MasterCycles(n) => {
                                            let goal_cycles = c.total_master_clocks() + n as u64;
                                            while *c.total_master_clocks() < goal_cycles {
                                                advance!(c, s);
                                            }
                                        }
                                        Scanlines(n) => (0..n).for_each(|_| {
                                            let mut hblank = c.ppu().is_in_hblank();
                                            while !(hblank && !c.ppu().is_in_hblank()) {
                                                hblank = c.ppu().is_in_hblank();
                                                advance!(c, s);
                                            }
                                        }),
                                        Instructions(instructions) => {
                                            (0..instructions).for_each(|_| {
                                                advance!(c, s);
                                            });
                                        }
                                        Frames(n) => (0..n).for_each(|_| {
                                            let mut v = c.ppu().is_in_vblank();
                                            while !(!v && c.ppu().is_in_vblank()) {
                                                v = c.ppu().is_in_vblank();
                                                advance!(c,s);
                                            }
                                        }),
                                        StartVBlank => {
                                            let mut vblank = c.ppu().is_in_vblank();
                                            while !(!vblank && c.ppu().is_in_vblank()) {
                                                vblank = c.ppu().is_in_vblank();
                                                advance!(c, s);
                                            }
                                        }
                                        EndVBlank => {
                                            let mut vblank = c.ppu().is_in_vblank();
                                            while !(vblank && !c.ppu().is_in_vblank()) {
                                                vblank = c.ppu().is_in_vblank();
                                                advance!(c, s);
                                            }
                                        }
                                    }
                                    update_ui!(c, s);
                                }
                                UpdateInputPorts(input_ports) => {
                                    *c.input_ports_mut() = input_ports;
                                }
                                LoadRom(bytes) => {
                                    *c = Console::with_cartridge(&bytes);
                                }
                                LoadSavestate(state) => {
                                    *c = state;
                                    c.ppu_mut().reset_vram_cache();
                                }
                                Reset => {
                                    c.reset();
                                }
                            };
                            update_ui!(c, s);
                            },
                            Err(_) => {}
                        }
                        // Calculate delta time
                        let now = Instant::now();
                        let dt = now - last_time;
                        last_time = now;
                        let s = settings.lock().unwrap().deref().clone();
                        // Advance emulator
                        if !s.is_paused {
                            let initial_master_cycles = console.lock().unwrap().total_master_clocks().clone();
                            while ((console.lock().unwrap().total_master_clocks() - initial_master_cycles) as f64)
                                < dt.as_micros() as f64 / 1_000_000.0 * MASTER_CLOCK_SPEED_HZ as f64
                            {
                                let mut c = console.lock().unwrap();
                                let vblank = c.ppu().is_in_vblank();
                                advance!(c, s);
                                // Update canvas if we just entered vblank
                                if vblank && !c.ppu().is_in_vblank() {
                                    update_ui!(c, s);
                                }
                            }
                            // Update audio
                            let samples = console.lock().unwrap().apu_mut().sample_queue();
                            let (a, b) = samples.as_slices();
                            audio.push_samples(a, s.volume);
                            audio.push_samples(b, s.volume);
                        }
                        // Sleep
                        thread::sleep(SLEEP_TIME);
                    }
            }))
            .expect("Unable to spawn thread");

        Engine {
            to_emu,
            console,
            settings,
            cpu_dis,
            apu_dis,
        }
    }

    pub fn update_settings(&mut self, settings: Settings) {
        *self.settings.lock().unwrap() = settings;
    }

    pub fn update(&mut self, command: Command) {
        self.to_emu
            .send(UpdateEmuPayload { command })
            .expect("Unable to send data to thread");
    }

    pub fn get_savestate(&self) -> Vec<u8> {
        let c = self.console.lock().unwrap().clone();
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
