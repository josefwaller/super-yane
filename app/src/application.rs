use std::{
    collections::VecDeque,
    fmt::Display,
    path::Path,
    thread::{self},
    time::{Duration, Instant},
};

use crate::widgets::{background_table, screen::Screen, text_table, vertical_table};
use iced::{
    Alignment::Center,
    Color, Element,
    Event::{self, Keyboard},
    Length, Padding, Subscription, Task, Theme, color, event,
    keyboard::{
        Event::{KeyPressed, KeyReleased},
        Key,
        key::Key::{Character, Named},
    },
    widget::{
        Column, button, canvas, checkbox, column, container, pick_list, row, scrollable, text_input,
    },
    window,
};
use itertools::Itertools;
use log::*;

use iced::widget::text;

use rfd::FileDialog;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use spc700::{OpcodeData as Spc700OpcodeData, format_address_modes};
use super_yane::{Console, InputPort, MASTER_CLOCK_SPEED_HZ};
use wavers::{Samples, write};
use wdc65816::{format_address_mode, opcode_data};

use crate::utils::utils::{hex_fmt, table_row};
use crate::{
    EmuState, apu_snapshot::ApuSnapshot, instruction_snapshot::InstructionSnapshot,
    widgets::ram::ram,
};
use std::sync::{Arc, Mutex};

pub const VOLUME: f32 = 5.0;
pub fn with_indent<'a, Message: 'a>(
    e: impl Into<Element<'a, Message>>,
) -> impl Into<Element<'a, Message>> {
    container(e.into()).padding(Padding::new(0.0).left(30))
}

pub const COLORS: [Color; 5] = [
    color!(0x98c2d4),
    color!(0xd49e98),
    color!(0x91c29c),
    color!(0xc2af91),
    color!(0xf06262),
];

#[derive(Debug, Clone, PartialEq)]
enum RamDisplay {
    WorkRam,
    VideoRam,
    ColorRam,
}
impl Display for RamDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RamDisplay::*;
        write!(
            f,
            "{}",
            match self {
                WorkRam => "WRAM",
                VideoRam => "VRAM",
                ColorRam => "CGRAM",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InstDisplay {
    Cpu,
    Apu,
}
impl Display for InstDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InstDisplay::*;
        write!(
            f,
            "{}",
            match self {
                Cpu => "CPU",
                Apu => "APU",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    NewFrame(),
    AdvanceInstructions(u32),
    AdvanceBreakpoint,
    AddBreakpoint(u8),
    RemoveBreakpoint(u8),
    ToggleVBlankBreakpoint(bool),
    SetOpcodeSearch(String),
    ToggleFutureStates(bool),
    /// Go back 1 state
    PreviousInstruction,
    PreviousBreakpoint,
    OnEvent(Event),
    ChangeVramPage(usize),
    ChangePaused(bool),
    SetRamDisplay(RamDisplay),
    SetInstDisplay(InstDisplay),
    LoadRom,
    Reset,
    ToggleLogCpu(bool),
    ToggleLogApu(bool),
    Record(bool),
    WindowClose(),
}

pub struct Application {
    console: Console,
    ram_offset: usize,
    is_paused: bool,
    breakpoint_opcodes: Vec<u8>,
    previous_breakpoint_states: VecDeque<Console>,
    previous_console: Box<Console>,
    previous_console_lag: u32,
    previous_states: VecDeque<Console>,
    opcode_search: String,
    vblank_breakpoint: bool,
    ignore_breakpoints: bool,
    emulate_future_states: bool,
    ram_display: RamDisplay,
    total_instructions: u32,
    previous_instruction_snapshots: VecDeque<InstructionSnapshot>,
    previous_apu_snapshots: VecDeque<ApuSnapshot>,
    inst_display: InstDisplay,
    log_cpu: bool,
    log_apu: bool,
    record: bool,
    samples: Vec<f32>,
    screen_data: [[u8; 4]; 256 * 240],
    emu_time: Duration,
    last_frame_time: Instant,
    channel: AudioQueue<f32>,
    state: Arc<Mutex<EmuState>>,
}

const NUM_BREAKPOINT_STATES: usize = 20;
const NUM_PREVIOUS_STATES: usize = 200;

impl Default for Application {
    fn default() -> Self {
        let default_console = Console::with_cartridge(include_bytes!("../roms/HelloWorld.sfc"));
        let sdl = sdl2::init().expect("Unable to init SDL");
        let audio = sdl.audio().unwrap();
        let channel: AudioQueue<f32> = audio
            .open_queue(
                None,
                &AudioSpecDesired {
                    freq: Some(32_000),
                    channels: Some(1),
                    samples: None,
                },
            )
            .unwrap();
        debug!("Channel spec is {:?}", channel.spec());
        let state = Arc::new(Mutex::new(EmuState::new(
            Some(Console::with_cartridge(include_bytes!(
                "../roms/HelloWorld.sfc"
            ))),
            0,
        )));
        let thread_state = Arc::clone(&state);

        thread::Builder::new()
            .name("Super Y.A.N.E. helper".to_string())
            .spawn(move || {
                loop {
                    let mut lock = thread_state.lock().unwrap();
                    let total_cycles = lock.total_cycles;
                    match lock.emu.as_mut() {
                        None => {}
                        Some(e) => {
                            while *e.total_master_clocks() < total_cycles {
                                e.advance_instructions(1);
                            }
                        }
                    }
                }
            })
            .expect("Unable to spawn thread");

        let console = state.lock().unwrap().emu.clone().unwrap().clone();

        Application {
            console,
            ram_offset: 0,
            is_paused: true,
            breakpoint_opcodes: vec![],
            previous_breakpoint_states: VecDeque::with_capacity(NUM_BREAKPOINT_STATES),
            opcode_search: String::new(),
            vblank_breakpoint: false,
            ignore_breakpoints: false,
            emulate_future_states: false,
            ram_display: RamDisplay::VideoRam,
            total_instructions: 0,
            previous_console: Box::new(default_console.clone()),
            previous_console_lag: 0,
            previous_states: VecDeque::with_capacity(500),
            previous_instruction_snapshots: VecDeque::with_capacity(NUM_PREVIOUS_STATES),
            previous_apu_snapshots: VecDeque::with_capacity(NUM_PREVIOUS_STATES),
            inst_display: InstDisplay::Cpu,
            log_cpu: false,
            log_apu: false,
            record: false,
            samples: vec![],
            screen_data: [[0; 4]; 256 * 240],
            emu_time: Duration::from_micros(0),
            last_frame_time: Instant::now(),
            channel,
            state,
        }
    }
}

impl Application {
    fn on_breakpoint(&mut self) {
        if !self.ignore_breakpoints {
            self.pause();
        }
    }
    fn pause(&mut self) {
        self.is_paused = true;
    }
    fn is_in_breakpoint(&mut self) -> bool {
        false
    }
    fn advance(&mut self) {
        if self.is_in_breakpoint() {
            self.previous_breakpoint_states
                .push_back(self.console.clone());
            if self.previous_breakpoint_states.len() > NUM_BREAKPOINT_STATES {
                self.previous_breakpoint_states.pop_front();
            }
        }
        let vblank = self.console.ppu().is_in_vblank();
        self.console.advance_instructions_with_hooks(
            1,
            &mut |c| {
                let snap = InstructionSnapshot::from(c);
                if self.log_cpu {
                    debug!("CPU_STATE {}", snap);
                }
                self.previous_instruction_snapshots.push_back(snap);
                if self.previous_instruction_snapshots.len() > NUM_PREVIOUS_STATES {
                    self.previous_instruction_snapshots.pop_front();
                }
            },
            &mut |c| {
                let snap = ApuSnapshot::from(c);
                if self.log_apu {
                    debug!(
                        "APU_STATE {} PORTS APU2CPU={:02X?} CPU2APU={:02X?}",
                        snap,
                        c.apu_to_cpu_reg(),
                        c.cpu_to_apu_reg()
                    );
                }
                self.previous_apu_snapshots.push_back(snap);
                if self.previous_apu_snapshots.len() > NUM_PREVIOUS_STATES {
                    self.previous_apu_snapshots.pop_front();
                }
            },
        );
        if !vblank && self.console.ppu().is_in_vblank() {
            self.screen_data = self
                .console
                .ppu()
                .screen_data_rgb()
                .map(|[r, g, b]| [r, g, b, 0xFF]);
        }
        self.total_instructions += 1;
        self.previous_console_lag += 1;
        while self.previous_console_lag > 500 {
            self.previous_console_lag -= 1;
        }
        if self.is_in_breakpoint() {
            self.on_breakpoint();
        }
    }
    fn on_key_change(&mut self, key: Key, value: bool) {
        let key_value: Option<(String, bool)> = match key {
            Character(c) => Some((c.to_string(), value)),
            Named(c) => {
                if c == iced::keyboard::key::Named::Space {
                    Some((' '.to_string(), value))
                } else {
                    None
                }
            }
            _ => None,
        };
        // let c = self.console.input_ports()[0];
        let c = self
            .state
            .lock()
            .unwrap()
            .emu
            .as_ref()
            .unwrap()
            .input_ports()[0];
        match c {
            InputPort::Empty => {}
            InputPort::StandardController {
                mut a,
                mut b,
                mut x,
                mut y,
                mut up,
                mut left,
                mut right,
                mut down,
                mut start,
                mut select,
                mut r,
                mut l,
            } => match key_value {
                Some((key, val)) => {
                    match key.as_str() {
                        "w" => up = val,
                        "a" => left = val,
                        "s" => down = val,
                        "d" => right = val,
                        "n" => y = val,
                        "m" => x = val,
                        " " => b = val,
                        "r" => start = val,
                        "f" => select = val,
                        "b" => a = val,
                        "q" => l = val,
                        "e" => r = val,
                        _ => {}
                    };
                    self.state
                        .lock()
                        .unwrap()
                        .emu
                        .as_mut()
                        .unwrap()
                        .input_ports_mut()[0] = InputPort::StandardController {
                        a,
                        b,
                        x,
                        y,
                        up,
                        left,
                        right,
                        down,
                        start,
                        select,
                        r,
                        l,
                    };
                }
                _ => {}
            },
        }
    }
    fn handle_input(&mut self, event: Event) -> Task<Message> {
        if let Keyboard(keyboard_event) = event {
            match keyboard_event {
                KeyPressed { key, .. } => self.on_key_change(key, true),
                KeyReleased { key, .. } => self.on_key_change(key, false),
                _ => {}
            };
        } else if let event::Event::Window(window_event) = event {
            match window_event {
                window::Event::CloseRequested => {
                    debug!("Closing application");
                    let samples = Samples::new(self.samples.clone().into_boxed_slice());
                    write(Path::new("./samples.wav"), &samples, 32_000, 1).unwrap();
                    return window::get_latest().and_then(|id| window::close(id));
                }
                _ => {}
            }
        }
        Task::none()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowClose() => {}
            Message::OnEvent(e) => {
                return self.handle_input(e);
            }
            Message::NewFrame() => {
                let ft = Instant::now();
                if !self.is_paused {
                    self.emu_time += ft - self.last_frame_time;
                }
                self.console = self.state.lock().unwrap().emu.clone().unwrap();
                self.state.lock().unwrap().total_cycles =
                    (MASTER_CLOCK_SPEED_HZ as f64 * self.emu_time.as_secs_f64()).floor() as u64;
                self.screen_data = self
                    .console
                    .ppu()
                    .screen_data_rgb()
                    .map(|[r, g, b]| [r, g, b, 0xFF]);
                self.last_frame_time = ft;
                let samples: Vec<f32> = self
                    .state
                    .lock()
                    .unwrap()
                    .emu
                    .as_mut()
                    .unwrap()
                    .apu_mut()
                    .sample_queue()
                    .into_iter()
                    .map(|s| VOLUME * s)
                    .collect();
                if self.record {
                    self.samples.extend_from_slice(samples.as_slice());
                }
                if self.channel.size() == 0 {
                    // error!("Empty queue");
                }
                if self.channel.size() > 20_000 {
                    self.channel.clear();
                }

                if samples
                    .iter()
                    .find(|x| **x > VOLUME || **x < -VOLUME)
                    .is_some()
                {
                    error!("INVALID SAMPLE");
                } else {
                    self.channel
                        .queue_audio(samples.as_slice())
                        .expect("Unable to enqueue audio");
                }
            }
            Message::AdvanceBreakpoint => {
                self.is_paused = false;
                self.ignore_breakpoints = false;
            }
            Message::PreviousBreakpoint => {
                //     if !self.previous_breakpoint_states.is_empty() {
                //         self.console = self.previous_breakpoint_states.pop_back().unwrap();
                //         self.is_paused = true;
                //     }
            }
            Message::PreviousInstruction => {
                if self.previous_states.len() > 0 {
                    // self.console = self.previous_states.pop_back().unwrap();
                    self.previous_console_lag -= 1;
                    self.is_paused = true;
                }
            }
            Message::AdvanceInstructions(num_instructions) => {
                (0..num_instructions).for_each(|_| {
                    self.advance();
                });
            }
            Message::ChangeVramPage(new_vram_page) => self.ram_offset = new_vram_page,
            Message::ChangePaused(p) => {
                if p {
                    self.pause();
                } else {
                    self.is_paused = false;
                    self.ignore_breakpoints = true;
                    self.channel.resume();
                }
            }
            Message::AddBreakpoint(b) => self.breakpoint_opcodes.push(b),
            Message::RemoveBreakpoint(b) => self.breakpoint_opcodes.retain(|o| *o != b),
            Message::Reset => {
                self.state.lock().unwrap().emu.as_mut().unwrap().reset();
                self.previous_states.clear();
            }
            Message::SetOpcodeSearch(s) => self.opcode_search = s,
            Message::ToggleVBlankBreakpoint(v) => self.vblank_breakpoint = v,
            Message::ToggleFutureStates(v) => self.emulate_future_states = v,
            Message::LoadRom => match FileDialog::new()
                .add_filter("Super FamiCon files", &["sfc"])
                .pick_file()
            {
                Some(p) => {
                    let bytes = std::fs::read(&p)
                        .expect(format!("Unable to read file '{:?}': ", p).as_str());
                    *self.state.lock().unwrap() = {
                        let console = Console::with_cartridge(&bytes);
                        EmuState::new(Some(console), 0)
                    };
                    self.emu_time = Duration::ZERO;
                    //     self.previous_console = Box::new(self.console.clone());
                    //     self.previous_console_lag = 0;
                    //     self.previous_states.clear();
                    //     self.total_instructions = 0;
                    //     self.previous_apu_snapshots.clear();
                    //     self.previous_instruction_snapshots.clear();
                    self.is_paused = true;
                }
                None => {}
            },
            Message::SetRamDisplay(d) => {
                self.ram_display = d;
                self.ram_offset = 0;
            }
            Message::SetInstDisplay(d) => self.inst_display = d,
            Message::ToggleLogApu(v) => self.log_apu = v,
            Message::ToggleLogCpu(v) => self.log_cpu = v,
            Message::Record(v) => {
                self.record = v;
            }
        };
        Task::none()
    }
    pub fn view(&self) -> Element<'_, Message> {
        column![
            row![
                scrollable(column![
                    self.cpu_data().into(),
                    self.apu_data().into(),
                    self.ppu_data().into(),
                    self.dma_data()
                ])
                .spacing(0)
                .width(Length::Shrink),
                container(column![
                    canvas(Screen {
                        frame_data: self.screen_data.as_flattened(),
                    })
                    .height(Length::Fill)
                    .width(Length::Fill),
                    row![
                        button("OPEN").on_press(Message::LoadRom),
                        button("RESET").on_press(Message::Reset),
                        button("|<<").on_press(Message::PreviousBreakpoint),
                        button("|<").on_press(Message::PreviousInstruction),
                        button(if self.is_paused { " >" } else { "||" })
                            .on_press(Message::ChangePaused(!self.is_paused)),
                        button(">|").on_press(Message::AdvanceInstructions(1)),
                        button(">>|").on_press(Message::AdvanceBreakpoint),
                        button(">>5,000").on_press(Message::AdvanceInstructions(5000)),
                        button(">>50,000").on_press(Message::AdvanceInstructions(50000)),
                    ],
                    row![text(format!(
                        "Total master cycles: {:08}, total APU cycles {:08}, total instructions: {:08}",
                        self.console.total_master_clocks().clone(),
                        self.console.total_apu_clocks().clone(),
                        self.total_instructions
                    ))]
                ])
                .style(|_| iced::widget::container::background(
                    iced::Background::Color(Color::BLACK)
                )),
                column![
                    container(self.instructions()).height(Length::Fill),
                    row![
                        checkbox("Log CPU", self.log_cpu).on_toggle(Message::ToggleLogCpu),
                        checkbox("Log APU", self.log_apu).on_toggle(Message::ToggleLogApu),
                        checkbox("Record", self.record).on_toggle(Message::Record)
                    ]
                ]
                .width(Length::Shrink)
            ]
            .spacing(10)
            .height(Length::Fill),
            row![self.ram_view().into(), self.breakpoints().into()]
                .spacing(10)
                .height(Length::Fixed(200.0)),
        ]
        .padding(10)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }
    fn ram_view(&self) -> impl Into<Element<Message>> {
        Column::with_children([
            pick_list(
                [
                    RamDisplay::VideoRam,
                    RamDisplay::WorkRam,
                    RamDisplay::ColorRam,
                ],
                Some(self.ram_display.clone()),
                Message::SetRamDisplay,
            )
            .into(),
            if self.ram_display == RamDisplay::ColorRam {
                ram(
                    &self.console.ppu().cgram,
                    self.ram_offset,
                    COLORS[1],
                    Color::WHITE,
                    color!(0xAAAAAA),
                    0,
                )
                .into()
            } else {
                ram(
                    match self.ram_display {
                        RamDisplay::VideoRam => &self.console.ppu().vram,
                        RamDisplay::WorkRam => self.console.ram().as_slice(),
                        RamDisplay::ColorRam => &[],
                    },
                    self.ram_offset,
                    match self.ram_display {
                        RamDisplay::VideoRam => COLORS[3],
                        RamDisplay::WorkRam => COLORS[2],
                        RamDisplay::ColorRam => COLORS[1],
                    },
                    Color::WHITE,
                    color!(0xAAAAAA),
                    match self.ram_display {
                        RamDisplay::VideoRam => 0,
                        RamDisplay::WorkRam => 0x7E0000,
                        RamDisplay::ColorRam => 0,
                    },
                )
                .into()
            },
        ])
    }
    fn breakpoints(&self) -> impl Into<Element<Message>> {
        column![
            Element::<Message>::from(
                text_input("Filter", &self.opcode_search).on_input(Message::SetOpcodeSearch)
            ),
            scrollable(Element::<Message>::from(Column::with_children(
                self.breakpoint_opcodes
                    .iter()
                    .map(|op| {
                        let data = opcode_data(*op, false, false);
                        checkbox(
                            format!("{} {} ({:02X})", data.name, data.addr_mode, op),
                            true,
                        )
                        .on_toggle(|_| Message::RemoveBreakpoint(*op))
                        .into()
                    })
                    .chain(
                        [checkbox("VBlank", self.vblank_breakpoint)
                            .on_toggle(Message::ToggleVBlankBreakpoint)
                            .into()]
                        .into_iter()
                    )
                    .chain(
                        (0..=255)
                            .filter(|op| !self.breakpoint_opcodes.contains(op))
                            .map(|op| opcode_data(op, false, false))
                            .filter(|data| self.opcode_search.is_empty()
                                || data
                                    .name
                                    .to_lowercase()
                                    .contains(&self.opcode_search.to_lowercase()))
                            .sorted()
                            .map(|data| {
                                checkbox(
                                    format!("{} {} ({:02X})", data.name, data.addr_mode, data.code),
                                    false,
                                )
                                .on_toggle(move |_| Message::AddBreakpoint(data.code))
                                .into()
                            })
                    )
            )))
            .width(Length::Fill)
        ]
    }

    fn cpu_data(&self) -> impl Into<Element<Message>> {
        let cpu = self.console.cpu().clone();
        let values = text_table(
            [
                ("C", cpu.c(), 4),
                ("X", cpu.x(), 4),
                ("Y", cpu.y(), 4),
                ("PBR", cpu.pbr.into(), 2),
                ("PC", cpu.pc, 2),
                ("DBR", cpu.dbr.into(), 2),
                ("D", cpu.dr(), 4),
                ("SP", cpu.s, 4),
                ("C 16:", cpu.c_true(), 4),
                ("P", cpu.p.to_byte(true).into(), 2),
                ("P actual", cpu.p.to_byte(false).into(), 2),
            ]
            .into_iter()
            .map(|(label, value, width)| {
                [
                    (label.to_string(), Some(COLORS[0])),
                    (format!("{:0width$X}", value, width = width), None),
                ]
            })
            .flatten(),
            2,
        );
        let p_table = text_table(
            [
                ("c", cpu.p.c),
                ("z", cpu.p.z),
                ("n", cpu.p.n),
                ("d", cpu.p.d),
                ("i", cpu.p.i),
                ("m", cpu.p.m),
                ("v", cpu.p.v),
                ("e", cpu.p.e),
                ("xb", cpu.p.xb),
            ]
            .into_iter()
            .map(|(label, value)| {
                [
                    (label.to_string(), Some(COLORS[1])),
                    (format!("{}", value), None),
                ]
            })
            .flatten(),
            2,
        );
        column![
            text("CPU").color(COLORS[4]),
            values.into(),
            with_indent(p_table).into(),
        ]
    }
    fn ppu_data(&self) -> impl Into<Element<'_, Message>> {
        macro_rules! ppu_val {
            ($label: expr, $field: ident, $format_str: expr) => {
                table_row!($label, self.console.ppu().$field, $format_str)
            };
            ($label: expr, $field: ident) => {
                ppu_val!($label, $field, "{}")
            };
        }
        let values = vec![
            ppu_val!("VBlank", vblank),
            ppu_val!("Forced VBlanking", forced_blanking),
            ppu_val!("Brightness", brightness, hex_fmt!()),
            ppu_val!("Background Mode", bg_mode),
            ppu_val!("Mosaic Size", mosaic_size, "{:X}px"),
            ppu_val!("BG3 Priority", bg3_prio),
            table_row!(
                "VRAM address (byte)",
                self.console.ppu().vram_addr,
                "{:06X}"
            ),
            table_row!(
                "VRAM address (word)",
                2 * self.console.ppu().vram_addr,
                "{:06X}"
            ),
            table_row!(
                "VRAM INC mode",
                self.console.ppu().vram_increment_mode,
                "{:?}"
            ),
        ];
        column![
            text("PPU").color(COLORS[4]),
            vertical_table(values, 150.0, 0),
            Column::with_children(self.console.ppu().backgrounds.iter().enumerate().map(
                |(i, b)| {
                    column![
                        text(format!("Background {}", i)).color(COLORS[0]),
                        with_indent(background_table(b)).into()
                    ]
                    .into()
                }
            )),
            Column::with_children(self.console.ppu().windows.iter().enumerate().map(|(i, w)| {
                column![
                    text(format!("Window {}", i)).color(COLORS[0]),
                    with_indent(text(format!(
                        "Left: {:02X} | Right: {:02X}",
                        w.left, w.right
                    )))
                    .into()
                ]
                .into()
            }))
        ]
    }
    fn apu_data(&self) -> impl Into<Element<'_, Message>> {
        let apu = self.console.apu().core;
        let values = text_table(
            [
                ("PC", apu.pc, 4),
                ("A", apu.a as u16, 2),
                ("X", apu.x as u16, 2),
                ("Y", apu.y as u16, 2),
            ]
            .into_iter()
            .map(|(label, value, width)| {
                [
                    (label.to_string(), Some(COLORS[0])),
                    (format!("{:0width$X}", value, width = width), None),
                ]
            })
            .flatten(),
            2,
        );
        let column_labels = ["", "R", "W"];
        let columns = core::array::from_fn(|i| (column_labels[i].to_string(), Some(COLORS[1])));
        let port_reads = text_table(
            [columns]
                .into_iter()
                .chain((0..4).into_iter().map(|i| {
                    [
                        (format!("{:02X}", 0xF4 + i).to_string(), Some(COLORS[1])),
                        (format!("{:02X}", self.console.cpu_to_apu_reg()[i]), None),
                        (format!("{:02X}", self.console.apu_to_cpu_reg()[i]), None),
                    ]
                }))
                .flatten(),
            3,
        );
        column![
            text("APU").color(COLORS[4]),
            values.into(),
            port_reads.into()
        ]
    }
    fn dma_data(&self) -> Column<'_, Message> {
        Column::with_children(
            self.console
                .dma_channels()
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    row![
                        text(i.to_string()),
                        vertical_table(
                            vec![
                                table_row!("Transfer Pattern", d.transfer_pattern, "{:?}"),
                                table_row!("Address Adjust Mode", d.adjust_mode, "{:?}"),
                                table_row!("Indirect", d.indirect, "{}"),
                                table_row!("Direction", d.direction, "{:?}"),
                                table_row!("Destination", d.dest_addr, "{:02X}"),
                                table_row!(
                                    "Source",
                                    d.src_bank as usize * 0x10000 + d.src_addr as usize,
                                    "{:06X}"
                                ),
                                table_row!("Bytes Remaining", d.byte_counter, "{:04X}")
                            ],
                            150.0,
                            0,
                        )
                    ]
                    .into()
                }),
        )
    }
    fn instructions(&self) -> impl Into<Element<Message>> {
        column![
            pick_list(
                [InstDisplay::Cpu, InstDisplay::Apu],
                Some(self.inst_display.clone()),
                Message::SetInstDisplay,
            ),
            container(match self.inst_display {
                InstDisplay::Apu => scrollable(Column::with_children(
                    self.previous_apu_snapshots
                        .iter()
                        .chain([ApuSnapshot::from(&self.console)].iter())
                        .enumerate()
                        .map(|(i, s)| {
                            let data = Spc700OpcodeData::from_opcode(s.opcode);
                            row![
                                text(format!("{:04X}", s.cpu.pc)),
                                text(format!("{:02X}", s.opcode)),
                                text(format!("{:4}", data.name)),
                                text(format!(
                                    "{:8}",
                                    format_address_modes(&data.addr_modes, &s.operands,)
                                )),
                            ]
                            .spacing(10)
                            .into()
                        })
                ))
                .width(Length::Fill)
                .anchor_bottom()
                .spacing(10),
                InstDisplay::Cpu => scrollable(Column::with_children(
                    self.previous_instruction_snapshots
                        .iter()
                        .chain([InstructionSnapshot::from(&self.console)].iter())
                        .enumerate()
                        .map(|(i, s)| {
                            let data =
                                opcode_data(s.opcode, s.cpu.p.a_is_16bit(), s.cpu.p.xy_is_16bit());
                            row![
                                text(format!("{:02X}", s.cpu.pbr)),
                                text(format!("{:04X}", s.cpu.pc)),
                                text(format!("{:02X}", s.opcode)),
                                text(data.name.to_string()).color(if i == NUM_PREVIOUS_STATES {
                                    Color::WHITE
                                } else {
                                    COLORS[0]
                                }),
                                text(format!(
                                    "{:6}",
                                    format_address_mode(data.addr_mode, &s.operands, data.bytes,)
                                )),
                            ]
                            .spacing(10)
                            .into()
                        }),
                ))
                .width(Length::Fill)
                .spacing(10)
                .anchor_bottom(),
            })
        ]
        .width(Length::Fixed(200.0))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::NewFrame()),
            event::listen().map(Message::OnEvent),
        ])
    }
    pub fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}
