use itertools::Itertools;
use std::{
    collections::VecDeque,
    fmt::Display,
    path::Path,
    time::{Duration, Instant},
};

use crate::{
    engine::{AdvanceAmount, AdvanceSettings, Engine},
    table::{cell, table},
    utils::vram_to_rgba,
    widgets::{background_table, screen::Screen, text_table, vertical_table},
};
use iced::{
    Alignment::{self, Center},
    Color, Element,
    Event::{self, Keyboard},
    Length, Padding, Subscription, Task, Theme, color, event,
    keyboard::{
        Event::{KeyPressed, KeyReleased},
        Key,
        key::Key::{Character, Named},
    },
    widget::{
        Column, Row, button, canvas, checkbox, column, container, pick_list, row, scrollable,
        space::horizontal,
    },
    window,
};
use log::*;

use iced::widget::text;

use rfd::FileDialog;
use sdl2::audio::AudioQueue;
use super_yane::{
    Console, InputPort, MASTER_CLOCK_SPEED_HZ, ppu::convert_8p8, utils::color_to_rgb_bytes,
};
use wavers::{Samples, write};

use crate::utils::utils::{hex_fmt, table_row};
use crate::{
    apu_snapshot::ApuSnapshot, instruction_snapshot::InstructionSnapshot, widgets::ram::ram,
};
use derive_new::new;

pub const VOLUME: f32 = 5.0;
pub fn with_indent<'a, Message: 'a>(
    e: impl Into<Element<'a, Message>>,
) -> impl Into<Element<'a, Message>> {
    container(e.into()).padding(Padding::new(0.0).left(30))
}

fn num_palettes(bpp: usize) -> usize {
    match bpp {
        2 => 64,
        4 => 16,
        8 => 1,
        _ => unreachable!("Invalid BPP {}", bpp),
    }
}

const NUM_TILES_X: usize = 16;
const NUM_TILES_Y: usize = 4;
const VRAM_IMAGE_WIDTH: usize = NUM_TILES_X * 9;
const VRAM_IMAGE_HEIGHT: usize = NUM_TILES_Y * 9;
const TILES_PER_PAGE: usize = NUM_TILES_X * NUM_TILES_Y;

pub const COLORS: [Color; 5] = [
    color!(0x98c2d4),
    color!(0xd49e98),
    color!(0x91c29c),
    color!(0xc2af91),
    color!(0xf06262),
];

#[derive(Debug, Clone, PartialEq)]
pub enum RamDisplay {
    WorkRam,
    /// Display VRAM as binary data
    VideoRamHex,
    /// Display VRAM as an image
    VideoRamTiles,
    ColorRam,
}
impl Display for RamDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RamDisplay::*;
        write!(
            f,
            "{}",
            match self {
                WorkRam => "WRAM".to_string(),
                VideoRamHex => "VRAM (Hex)".to_string(),
                VideoRamTiles => format!("VRAM (Tiles)"),
                ColorRam => "CGRAM".to_string(),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstDisplay {
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
#[derive(Default, PartialEq, Debug, Clone)]
pub enum InfoDisplay {
    #[default]
    General,
    Oam,
}
impl Display for InfoDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InfoDisplay::*;
        write!(
            f,
            "{}",
            match self {
                General => "General",
                Oam => "Oam",
            }
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    NewFrame(),
    AdvanceInstructions(u32),
    AdvanceFrames(u32),
    OnEvent(Event),
    SetRamPage(usize),
    ChangePaused(bool),
    SetRamDisplay(RamDisplay),
    SetVramBpp(usize),
    SetVramPalette(usize),
    SetVramDirectColor(bool),
    SetInstDisplay(InstDisplay),
    SetInfoDisplay(InfoDisplay),
    LoadRom,
    LoadSavestate,
    CreateSavestate,
    Reset,
    ToggleLogCpu(bool),
    ToggleLogApu(bool),
    Record(bool),
    OnClose(window::Id),
}

#[derive(new)]
pub struct Program {
    #[new(value = "0")]
    /// Which page of RAM we are viewing in the ram viewer
    ram_page: usize,
    #[new(value = "true")]
    is_paused: bool,
    #[new(value = "RamDisplay::WorkRam")]
    ram_display: RamDisplay,
    /// When showing VRAM as an image, how many BPP to show for the image
    #[new(value = "2")]
    vram_bpp: usize,
    /// When showing VRAM as an image, what palette to use
    #[new(value = "0")]
    vram_palette: usize,
    /// When showing VRAM as an image, whether to use direct color
    #[new(value = "false")]
    vram_direct_color: bool,
    /// Cached VRAM RGBA data for rendering
    #[new(value = "[[0; 4]; VRAM_IMAGE_WIDTH * VRAM_IMAGE_HEIGHT]")]
    vram_rgba_data: [[u8; 4]; VRAM_IMAGE_WIDTH * VRAM_IMAGE_HEIGHT],
    /// Cached OAM sprite data for rendering
    #[new(value = "Box::new([[[0;4]; 64 * 64]; 128])")]
    oam_sprite_rgba_data: Box<[[[u8; 4]; 64 * 64]; 128]>,
    #[new(value = "0")]
    total_instructions: u32,
    #[new(value = "VecDeque::new()")]
    previous_instruction_snapshots: VecDeque<InstructionSnapshot>,
    #[new(value = "VecDeque::new()")]
    previous_apu_snapshots: VecDeque<ApuSnapshot>,
    #[new(value = "InstDisplay::Cpu")]
    inst_display: InstDisplay,
    #[new(value = "InfoDisplay::General")]
    info_display: InfoDisplay,
    #[new(value = "false")]
    log_apu: bool,
    #[new(value = "false")]
    record: bool,
    #[new(value = "Vec::new()")]
    samples: Vec<f32>,
    #[new(value = "Duration::ZERO")]
    emu_time: Duration,
    #[new(value = "Instant::now()")]
    last_frame_time: Instant,
    channel: AudioQueue<f32>,
    #[new(value = "Engine::new()")]
    pub engine: Engine,
    #[new(value = "AdvanceSettings::default()")]
    settings: AdvanceSettings,
}

impl Program {
    /// Pauses the emulation
    fn pause(&mut self) {
        self.is_paused = true;
        self.channel.pause();
    }
    /// Handles key input
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
        let c = self.engine.input_ports[0];
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
                    self.engine.input_ports[0] = InputPort::StandardController {
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
        }
        Task::none()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OnClose(id) => {
                debug!("Closing application");
                let samples = Samples::new(self.samples.clone().into_boxed_slice());
                write(Path::new("./samples.wav"), &samples, 32_000, 1).unwrap();
                let disassembly_lines = self
                    .engine
                    .disassembly
                    .iter()
                    .map(|(pc, i)| format!("{:06X}\t{}", pc, i))
                    .join("\n");
                std::fs::write(Path::new("./disassembly.txt"), &disassembly_lines).unwrap();
                return window::latest().and_then(window::close);
            }
            Message::OnEvent(e) => {
                return self.handle_input(e);
            }
            Message::AdvanceFrames(n) => {
                self.engine
                    .advance_amount(AdvanceAmount::Frames(n as usize), self.settings.clone());
            }
            Message::NewFrame() => {
                let ft = Instant::now();
                if !self.is_paused {
                    self.emu_time += ft - self.last_frame_time;
                }
                if !self.is_paused {
                    self.engine
                        .advance_dt(ft - self.last_frame_time, self.settings.clone());
                }
                self.last_frame_time = ft;
                self.engine.on_frame();
                vram_to_rgba(
                    self.engine.console(),
                    (NUM_TILES_X, NUM_TILES_Y),
                    self.vram_bpp,
                    self.ram_page * TILES_PER_PAGE,
                    self.vram_palette,
                    self.vram_direct_color,
                    1,
                    &mut self.vram_rgba_data,
                );
                // Update sprite graphics
                (0..128).for_each(|i| {
                    let spr = &self.engine.console().ppu().oam_sprites[i];
                    let size = self.engine.console().ppu().oam_sizes[spr.size_select];
                    vram_to_rgba(
                        self.engine.console(),
                        (size.0 / 8, size.1 / 8),
                        4,
                        self.engine.console().ppu().sprite_tile_slice_addr(spr, 0) / (4 * 8),
                        // Sprites use the last 8 palettes
                        8 + spr.palette_index,
                        false,
                        0,
                        &mut self.oam_sprite_rgba_data[i],
                    );
                });
                let samples: Vec<f32> = self
                    .engine
                    .swap_samples()
                    .into_iter()
                    .map(|s| VOLUME * s)
                    .collect();
                if self.record {
                    self.samples.extend_from_slice(samples.as_slice());
                }
                if self.channel.size() == 0 {
                    // error!("Empty queue");
                }
                if self.channel.size() > 32_000 {
                    error!("Channel too big");
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
            Message::AdvanceInstructions(num_instructions) => {
                self.engine.advance_amount(
                    AdvanceAmount::Instructions(num_instructions),
                    self.settings.clone(),
                );
            }
            Message::SetRamPage(ram_page) => {
                const BYTES_PER_PAGE: usize = 0x20 * 8;
                let bytes_per_tile = 8 * self.vram_bpp;
                self.ram_page = ram_page.min(match self.ram_display {
                    RamDisplay::WorkRam => self.engine.console().ram().len() / BYTES_PER_PAGE,
                    RamDisplay::VideoRamHex => {
                        self.engine.console().ppu().vram.len() / BYTES_PER_PAGE
                    }
                    RamDisplay::ColorRam => {
                        self.engine.console().ppu().cgram.len() * 2 / BYTES_PER_PAGE
                    }
                    RamDisplay::VideoRamTiles => {
                        self.engine.console().ppu().vram.len() / bytes_per_tile / TILES_PER_PAGE
                    }
                });
            }
            Message::ChangePaused(p) => {
                if p {
                    self.pause();
                } else {
                    self.is_paused = false;
                    self.channel.resume();
                }
            }
            Message::Reset => {
                self.engine.reset();
            }
            Message::LoadRom => match FileDialog::new()
                .add_filter("Super FamiCon files", &["sfc"])
                .pick_file()
            {
                Some(p) => {
                    let bytes = std::fs::read(&p)
                        .expect(format!("Unable to read file '{:?}': ", p).as_str());
                    self.engine.load_rom(&bytes);
                    self.is_paused = true;
                    self.emu_time = Duration::ZERO;
                }
                None => {
                    self.last_frame_time = Instant::now();
                }
            },
            Message::LoadSavestate => match FileDialog::new()
                .add_filter("Super YANE savestate files", &["sy.bin"])
                .pick_file()
            {
                Some(p) => {
                    let bytes =
                        std::fs::read(&p).expect(format!("Unable to read file '{:?}'", p).as_str());
                    self.engine.load_savestate(&bytes);
                    self.is_paused = true;
                    self.emu_time = Duration::from_micros(
                        1_000_000 * self.engine.console().total_master_clocks()
                            / MASTER_CLOCK_SPEED_HZ,
                    );
                }
                None => {}
            },
            Message::CreateSavestate => {
                let bytes = serde_brief::to_vec(&self.engine.console()).unwrap();
                let filename = "./savestatenew.sy.bin";
                std::fs::write(filename, bytes).unwrap();
            }
            Message::SetRamDisplay(d) => {
                self.ram_display = d;
                self.ram_page = 0;
            }
            Message::SetVramBpp(bpp) => {
                let factor = self.vram_bpp as f32 / bpp as f32;
                self.ram_page = (self.ram_page as f32 * factor).floor() as usize;
                self.vram_bpp = bpp;
                if self.vram_palette >= num_palettes(bpp) {
                    self.vram_palette = 0;
                }
            }
            Message::SetVramPalette(palette) => self.vram_palette = palette,
            Message::SetVramDirectColor(dc) => self.vram_direct_color = dc,
            Message::SetInstDisplay(d) => self.inst_display = d,
            Message::ToggleLogApu(v) => self.log_apu = v,
            Message::ToggleLogCpu(v) => self.settings.log_cpu = v,
            Message::Record(v) => {
                self.record = v;
            }
            Message::SetInfoDisplay(v) => self.info_display = v,
        };
        Task::none()
    }
    pub fn info_display(&self) -> Element<'_, Message> {
        match self.info_display {
            InfoDisplay::General => column![
                self.cpu_data().into(),
                self.apu_data().into(),
                self.ppu_data().into(),
                self.dma_data()
            ]
            .into(),
            InfoDisplay::Oam => {
                return table::<7usize, Message>(
                    [" #", "( x, y)", "X", "Tile", "Name", "Size", "Sprite"],
                    self.engine
                        .console()
                        .ppu()
                        .oam_sprites
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let size = self.engine.console().ppu().oam_sizes[s.size_select];
                            [
                                cell(format!("{:02X}", i)).into(),
                                cell(format!("({:02X}, {:02X})", s.x, s.y)).into(),
                                cell(format!("{}", u8::from(s.msb_x))).into(),
                                cell(format!("{:02X}", s.tile_index)).into(),
                                cell(format!("{:01}", s.name_select)).into(),
                                cell(format!("{:?}", size)).into(),
                                canvas(Screen::new(
                                    self.oam_sprite_rgba_data[i][0..(size.0 * size.1)]
                                        .as_flattened(),
                                    size.0 as u32,
                                    size.1 as u32,
                                ))
                                .into(),
                            ]
                        }),
                )
                .into();
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        column![
            row![
                column![
                    pick_list(
                        [InfoDisplay::General, InfoDisplay::Oam],
                        Some(self.info_display.clone()),
                        Message::SetInfoDisplay,
                    ),
                    scrollable(self.info_display())
                    .spacing(0)
                ],
                container(column![
                    canvas(Screen {
                        rgba_data: self.engine.prev_frame_data.as_flattened(),
                        width: 256,
                        height: 240
                    })
                    .height(Length::Fill)
                    .width(Length::Fill),
                    row![
                        button("OPEN ROM").on_press(Message::LoadRom),
                        button(if self.is_paused { "PLAY" } else { "PAUSE" })
                            .on_press(Message::ChangePaused(!self.is_paused)),
                        button("NEW SAVESTATE").on_press(Message::CreateSavestate),
                        button("LOAD SAVESTATE").on_press(Message::LoadSavestate),
                        button("RESET").on_press(Message::Reset),
                        button("ADVANCE SCANLINE").on_press(Message::AdvanceInstructions(1364)),
                        button("ADVANCE FRAME").on_press(Message::AdvanceFrames(1)),
                        button("ADVANCE 500").on_press(Message::AdvanceInstructions(500))
                    ],
                    row![text(format!(
                        "Total master cycles: {:08}, total APU cycles {:08}, total instructions: {:08}",
                        self.engine.console().total_master_clocks().clone(),
                        self.engine.console().total_apu_clocks().clone(),
                        self.total_instructions
                    ))]
                ])
                .style(|_| iced::widget::container::background(
                    iced::Background::Color(Color::BLACK)
                )),
                column![
                    container(self.disassembly()).height(Length::Fill),
                    row![
                        text("Log CPU"),
                        checkbox(self.settings.log_cpu).on_toggle(Message::ToggleLogCpu),
                        text("Log APU"),
                        checkbox(self.log_apu).on_toggle(Message::ToggleLogApu),
                        text("Record"),
                        checkbox(self.record).on_toggle(Message::Record)
                    ]
                ]
                .width(Length::Shrink)
            ]
            .spacing(10)
            .height(Length::Fill),
            row![self.ram_view().into()]
                .spacing(10)
                .height(Length::Fixed(200.0)),
        ]
        .padding(10)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }
    fn ram_view(&self) -> impl Into<Element<Message>> {
        let tile_elements: Element<Message> = if self.ram_display == RamDisplay::VideoRamTiles {
            let n = num_palettes(self.vram_bpp);
            row![
                text("BPP:"),
                pick_list([2, 4, 8], Some(self.vram_bpp.clone()), Message::SetVramBpp),
                text("Palette:"),
                pick_list(
                    (0..n).collect::<Vec<usize>>(),
                    Some(self.vram_palette),
                    Message::SetVramPalette
                ),
                text("Use Direct Color:"),
                checkbox(self.vram_direct_color).on_toggle(Message::SetVramDirectColor)
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            horizontal().into()
        };
        Column::with_children([
            row![
                pick_list(
                    [
                        RamDisplay::VideoRamHex,
                        RamDisplay::VideoRamTiles,
                        RamDisplay::WorkRam,
                        RamDisplay::ColorRam,
                    ],
                    Some(self.ram_display.clone()),
                    Message::SetRamDisplay,
                ),
                button("Prev Page").on_press(Message::SetRamPage(self.ram_page.saturating_sub(1))),
                button("Next Page").on_press(Message::SetRamPage(self.ram_page + 1)),
                tile_elements
            ]
            .spacing(10)
            .into(),
            match self.ram_display {
                RamDisplay::ColorRam => ram(
                    &self.engine.console().ppu().cgram,
                    self.ram_page,
                    COLORS[1],
                    Color::WHITE,
                    color!(0xAAAAAA),
                    0,
                )
                .into(),
                RamDisplay::WorkRam => ram(
                    &self.engine.console().ram().as_slice(),
                    self.ram_page,
                    COLORS[2],
                    Color::WHITE,
                    color!(0xAAAAAA),
                    0x7E0000,
                )
                .into(),
                RamDisplay::VideoRamHex => ram(
                    &self.engine.console().ppu().vram,
                    self.ram_page,
                    COLORS[3],
                    Color::WHITE,
                    color!(0xAAAAAA),
                    0,
                )
                .into(),
                RamDisplay::VideoRamTiles => row![
                    Column::with_children([text("Index/Addr").into()].into_iter().chain(
                        (0..NUM_TILES_Y).map(|i| {
                            let tile_index =
                                (NUM_TILES_X * NUM_TILES_Y) * self.ram_page + NUM_TILES_X * i;
                            text(format!(
                                "{:03X}/0x{:04X}",
                                tile_index,
                                tile_index * 8 * self.vram_bpp
                            ))
                            .height(Length::Fill)
                            .align_y(Alignment::Center)
                            .into()
                        })
                    )),
                    column![
                        Row::with_children((0..NUM_TILES_X).map(|x| {
                            text(format!("+{:01X}/{:02X}", x, x * 8 * self.vram_bpp))
                                .width(Length::Fill)
                                .align_x(Alignment::Center)
                                .into()
                        })),
                        canvas(Screen {
                            rgba_data: self.vram_rgba_data.as_flattened(),
                            width: VRAM_IMAGE_WIDTH as u32,
                            height: VRAM_IMAGE_HEIGHT as u32,
                        })
                        .width(Length::Fill)
                        .height(Length::Fill)
                    ]
                ]
                .spacing(10)
                .into(),
            },
        ])
    }

    fn cpu_data(&self) -> impl Into<Element<Message>> {
        let cpu = self.engine.console().cpu().clone();
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
                table_row!($label, self.engine.console().ppu().$field, $format_str)
            };
            ($label: expr, $field: ident) => {
                ppu_val!($label, $field, "{:?}")
            };
        }
        macro_rules! mat_val {
            ($field: ident) => {
                text(format!(
                    "{:7.2}",
                    convert_8p8(self.engine.console().ppu().matrix.$field)
                ))
            };
        }
        macro_rules! mat_val_raw {
            ($field: ident) => {
                text(format!("{:04X}", self.engine.console().ppu().matrix.$field))
            };
        }
        let values = vec![
            (
                "Dot",
                text(format!(
                    "{}, {}",
                    self.engine.console().ppu().cursor_x(),
                    self.engine.console().ppu().cursor_y()
                ))
                .into(),
            ),
            ppu_val!("VBlank", vblank),
            ppu_val!("Forced VBlanking", forced_blanking),
            ppu_val!("Brightness", brightness, hex_fmt!()),
            ppu_val!("Background Mode", bg_mode),
            ppu_val!("Mosaic Size", mosaic_size, "{:X}px"),
            ppu_val!("BG3 Priority", bg3_prio),
            table_row!(
                "VRAM address (byte)",
                self.engine.console().ppu().vram_addr,
                "{:06X}"
            ),
            table_row!(
                "VRAM address (word)",
                2 * self.engine.console().ppu().vram_addr,
                "{:06X}"
            ),
            table_row!(
                "VRAM INC mode",
                self.engine.console().ppu().vram_increment_mode,
                "{:?}"
            ),
            ppu_val!("OAM Nametable Addr (word)", oam_name_addr, "{:04X}"),
            table_row!(
                "OAM Nametable Addr (byte)",
                self.engine.console().ppu().oam_name_addr * 2,
                "{:04X}"
            ),
            ppu_val!("OAM Nametable Select (word)", oam_name_select, "{:04X}"),
            table_row!(
                "OAM Nametable Select (byte)",
                self.engine.console().ppu().oam_name_select * 2,
                "{:04X}"
            ),
            ppu_val!("Color math SRC", color_math_src),
            ppu_val!("Fixed Color", fixed_color),
            ppu_val!("Color Window Main", color_window_main_region),
            ppu_val!("Color Window Sub", color_window_sub_region),
            (
                "M7 Matrix",
                row![
                    column![mat_val!(a), mat_val!(c)],
                    column![mat_val!(b), mat_val!(d)]
                ]
                .padding(3)
                .into(),
            ),
            (
                "M7 Matrix Raw",
                row![
                    column![mat_val_raw!(a), mat_val_raw!(c)].spacing(1),
                    column![mat_val_raw!(b), mat_val_raw!(d)].spacing(1)
                ]
                .padding(3)
                .into(),
            ),
            ppu_val!("M7 H off", m7_h_off, "{}"),
            ppu_val!("M7 V off", m7_v_off, "{}"),
            ppu_val!("M7 Repeat", m7_repeat),
            ppu_val!("M7 Fill", m7_fill, "{:?}"),
            ppu_val!("M7 Flip H", m7_flip_h),
            ppu_val!("M7 Flip V", m7_flip_v),
        ];
        column![
            text("PPU").color(COLORS[4]),
            vertical_table(values, 150.0, 0),
            Column::with_children(
                self.engine
                    .console()
                    .ppu()
                    .backgrounds
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        column![
                            text(format!("Background {}", i)).color(COLORS[0]),
                            with_indent(background_table(b)).into()
                        ]
                        .into()
                    })
            ),
            Column::with_children(self.engine.console().ppu().windows.iter().enumerate().map(
                |(i, w)| {
                    column![
                        text(format!("Window {}", i)).color(COLORS[0]),
                        with_indent(text(format!(
                            "Left: {:02X} | Right: {:02X}",
                            w.left, w.right
                        )))
                        .into(),
                        text(format!("Color enabled: {}", w.enabled_color)),
                        text(format!("Color inverted: {}", w.invert_color)),
                    ]
                    .into()
                }
            ))
        ]
    }
    fn apu_data(&self) -> impl Into<Element<'_, Message>> {
        let apu = self.engine.console().apu().core;
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
                        (
                            format!("{:02X}", self.engine.console().cpu_to_apu_reg()[i]),
                            None,
                        ),
                        (
                            format!("{:02X}", self.engine.console().apu_to_cpu_reg()[i]),
                            None,
                        ),
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
        Column::with_children(self.engine.console().dma_channels().iter().enumerate().map(
            |(i, d)| {
                row![
                    text(i.to_string()),
                    vertical_table(
                        vec![
                            table_row!("Is HDMA", d.is_hdma(), "{}"),
                            table_row!("Transfer Pattern", d.transfer_pattern(), "{:?}"),
                            table_row!("Address Adjust Mode", d.adjust_mode, "{:?}"),
                            table_row!("Indirect", d.indirect, "{}"),
                            table_row!("Direction", d.direction, "{:?}"),
                            table_row!("Destination", d.dest_addr, "{:02X}"),
                            table_row!(
                                "Source",
                                d.src_bank as usize * 0x10000 + d.src_addr as usize,
                                "{:06X}"
                            ),
                            table_row!("Bytes Remaining", d.byte_counter, "{:04X}"),
                            table_row!("Indirect data address", d.indirect_data_addr, "{:04X}"),
                            table_row!("HDMA Table addr", d.hdma_table_addr, "{:04X}")
                        ],
                        150.0,
                        0,
                    )
                ]
                .into()
            },
        ))
    }
    fn disassembly(&self) -> impl Into<Element<Message>> {
        scrollable(Column::with_children(self.engine.disassembly.iter().map(
            |(pc, inst)| {
                row![
                    text(format!("{:06X}", pc)).color(if self.engine.console().pc() == *pc {
                        color!(0xFF0000)
                    } else {
                        color!(0xFFFFFF)
                    }),
                    text(inst)
                ]
                .spacing(20)
                .into()
            },
        )))
        .width(Length::Fixed(200.0))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::NewFrame()),
            event::listen().map(Message::OnEvent),
            window::close_requests().map(Message::OnClose),
        ])
    }
    pub fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}
