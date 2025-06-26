use std::{collections::VecDeque, string::FromUtf16Error};

use crate::widgets::text_table;
use iced::{
    Alignment::{self, Center},
    Color, Element, Event, Font,
    Length::{self, FillPortion},
    Padding, Renderer, Subscription, Theme,
    alignment::Horizontal,
    color, event,
    widget::{
        Column, Container, Row, Scrollable, Slider, button, column, container, horizontal_space,
        image::{FilterMethod, Handle, Image},
        keyed_column, pick_list, row, scrollable, slider,
        text::IntoFragment,
        vertical_space,
    },
    window,
};
use log::*;

use iced::widget::text;

use super_yane::{Console, ppu::Background};
use wdc65816::{format_address_mode, opcode_data};

use crate::widgets::ram::ram;

macro_rules! hex_fmt {
    () => {
        "0x{:04X}"
    };
}

macro_rules! table_row {
    ($label: expr, $field: expr, $format_str: expr) => {
        ($label, text(format!($format_str, $field)).into())
    };
    ($label: expr, $field: ident) => {
        ppu_val!($label, $field, "{}")
    };
}

pub fn background_table(background: &Background) -> impl Into<Element<'_, Message>> {
    let b = background;
    let rows = vec![
        ("Main screen enabled", b.main_screen_enable.into()),
        ("Sub screen enabled", b.sub_screen_enable.into()),
        ("Tile size", b.tile_size),
        ("Mosaic", b.mosaic.into()),
        ("Tilemap Address (Byte)", b.tilemap_addr as u32),
        ("Tilemap Address (Word)", 2 * b.tilemap_addr as u32),
        ("CHR Address", b.chr_addr as u32),
        ("H offset", b.h_off),
        ("V offset", b.v_off),
        ("# H Tilemaps", b.num_horz_tilemaps),
        ("# V Tilemaps", b.num_vert_tilemaps),
    ];
    text_table(
        rows.into_iter()
            .map(|(s, v)| [(s.to_string(), Some(COLORS[1])), (format!("{}", v), None)])
            .flatten(),
        2,
    )
}

pub const COLORS: [Color; 5] = [
    color!(0x98c2d4),
    color!(0xd49e98),
    color!(0x91c29c),
    color!(0xc2af91),
    color!(0xf06262),
];
pub fn vertical_table<'a>(
    values: Vec<(impl Into<String>, Element<'a, Message>)>,
    header_width: f32,
    depth: usize,
) -> Element<'a, Message> {
    use iced::widget::keyed::Column;
    Column::with_children(values.into_iter().enumerate().map(|(i, (header, value))| {
        (
            i,
            row![
                text(header.into())
                    .width(Length::Fixed(header_width))
                    .color(COLORS[depth]),
                value
            ]
            .spacing(10)
            .into(),
        )
    }))
    // .width(Length::Fill)
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    NewFrame(),
    AdvanceInstructions(u32),
    /// Advance until hitting a certain opcode
    AdvanceUntilOpcode(u8),
    /// Go back 1 state
    Revert,
    // OnEvent(Event),
    ChangeVramPage(usize),
    ChangePaused(bool),
}

pub struct Application {
    console: Console,
    vram_offset: usize,
    is_paused: bool,
    breakpoint_opcode: Option<u8>,
    previous_states: VecDeque<Console>,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            console: Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc")),
            vram_offset: 0,
            is_paused: true,
            breakpoint_opcode: None,
            previous_states: VecDeque::new(),
        }
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            // Message::OnEvent(e) => {}
            Message::NewFrame() => {
                if !self.is_paused {
                    // Todo: Determine how many per frame
                    for _ in 0..100 {
                        self.previous_states.push_back(self.console.clone());
                        if self.previous_states.len() > 100 {
                            self.previous_states.pop_front();
                        }
                        self.console.advance_instructions(1);
                        let op = self.console.cartridge.read_byte(
                            self.console.cpu.pbr as usize * 0x10000 + self.console.cpu.pc as usize,
                        );
                        // Always break on BRKs
                        if op == 0x00 {
                            self.is_paused = true;
                            break;
                        }
                        match self.breakpoint_opcode {
                            Some(o) => {
                                if o == op {
                                    self.is_paused = true;
                                    break;
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
            Message::AdvanceUntilOpcode(opcode) => {
                self.is_paused = false;
                self.breakpoint_opcode = Some(opcode);
            }
            Message::Revert => {
                if self.previous_states.len() > 0 {
                    self.console = self.previous_states[self.previous_states.len() - 1].clone();
                    self.previous_states.pop_back();
                }
            }
            Message::AdvanceInstructions(num_instructions) => {
                self.previous_states.push_back(self.console.clone());
                self.console.advance_instructions(num_instructions);
            }
            Message::ChangeVramPage(new_vram_page) => self.vram_offset = new_vram_page,
            Message::ChangePaused(p) => {
                self.breakpoint_opcode = None;
                self.is_paused = p;
            }
        }
        while self.previous_states.len() > 100 {
            self.previous_states.pop_front();
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let data = self.console.ppu.screen_buffer.map(|color| {
            [
                ((color & 0x001F) << 3) as u8,
                ((color & 0x03E0) >> 2) as u8,
                ((color & 0x7C00) >> 7) as u8,
                0xFF,
            ]
        });
        column![
            row![
                scrollable(column![
                    self.cpu_data(),
                    self.ppu_data().into(),
                    self.dma_data()
                ])
                .spacing(0)
                .width(Length::Shrink),
                column![
                    Image::new(Handle::from_rgba(256, 240, data.as_flattened().to_vec()))
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .content_fit(iced::ContentFit::Contain)
                        .filter_method(FilterMethod::Nearest),
                    row![
                        button("|<").on_press(Message::Revert),
                        button(if self.is_paused { " >" } else { "||" })
                            .on_press(Message::ChangePaused(!self.is_paused)),
                        button(">|").on_press(Message::AdvanceInstructions(1)),
                        button("To BRK")
                            .on_press(Message::AdvanceUntilOpcode(wdc65816::opcodes::ADC_DIX))
                    ],
                ],
                scrollable(self.next_instructions())
                    .spacing(10)
                    .width(Length::Shrink)
            ]
            .spacing(10)
            .height(Length::Fill),
            row![ram(&self.console.ppu.vram, self.vram_offset, COLORS[3])]
                .spacing(10)
                .height(Length::Fixed(200.0))
        ]
        .padding(10)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }

    fn cpu_data(&self) -> Column<'_, Message> {
        let cpu = &self.console.cpu;
        let values = vec![
            table_row!("C", cpu.c(), "{:04X}"),
            table_row!("X", cpu.x(), "{:04X}"),
            table_row!("Y", cpu.y(), "{:04X}"),
            table_row!("PBR", cpu.pbr, "{:02X}"),
            table_row!("PC", cpu.pc, "{:04X}"),
            table_row!("DBR", cpu.dbr, "{:02X}"),
            table_row!("D", cpu.d, "{:04X}"),
            table_row!("SP", cpu.s, "{:04X}"),
            (
                "P",
                vertical_table(
                    vec![
                        table_row!("c", cpu.p.c, "{}"),
                        table_row!("z", cpu.p.z, "{}"),
                        table_row!("n", cpu.p.n, "{}"),
                        table_row!("d", cpu.p.d, "{}"),
                        table_row!("i", cpu.p.i, "{}"),
                        table_row!("m", cpu.p.m, "{}"),
                        table_row!("v", cpu.p.v, "{}"),
                        table_row!("e", cpu.p.e, "{}"),
                        table_row!("xb", cpu.p.xb, "{}"),
                    ],
                    20.0,
                    1,
                ),
            ),
        ];
        column![
            text("CPU").color(COLORS[4]),
            vertical_table(values, 50.0, 0)
        ]
    }
    fn ppu_data(&self) -> impl Into<Element<'_, Message>> {
        macro_rules! ppu_val {
            ($label: expr, $field: ident, $format_str: expr) => {
                table_row!($label, self.console.ppu.$field, $format_str)
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
            table_row!("VRAM address (byte)", self.console.ppu.vram_addr, "{:06X}"),
            table_row!(
                "VRAM address (word)",
                2 * self.console.ppu.vram_addr,
                "{:06X}"
            ),
            table_row!(
                "VRAM INC mode",
                self.console.ppu.vram_increment_mode,
                "{:?}"
            ),
        ];
        column![
            text("PPU").color(COLORS[4]),
            vertical_table(values, 150.0, 0),
            Column::with_children(
                self.console
                    .ppu
                    .backgrounds
                    .iter()
                    .enumerate()
                    .map(|(i, b)| column![
                        text(format!("Background {}", i)).color(COLORS[0]),
                        container(background_table(b).into()).padding(Padding::new(0.).left(30))
                    ]
                    .into())
            ),
        ]
    }
    fn dma_data(&self) -> Column<'_, Message> {
        Column::with_children(self.console.dma_channels.iter().enumerate().map(|(i, d)| {
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
        }))
    }
    /// Represent a console's state (PC, PBR, opcode) as a table row
    fn console_table_row(
        &self,
        console: &Console,
        color: Option<Color>,
    ) -> Vec<(String, Option<Color>)> {
        let addr = console.cpu.pbr as usize * 0x10000 + console.cpu.pc as usize;
        let opcode = console.cartridge.read_byte(addr);
        let data = opcode_data(
            opcode,
            console.cpu.p.a_is_16bit(),
            console.cpu.p.xy_is_16bit(),
        );
        let c = &console.cartridge;
        [
            format!("{:02X}", console.cpu.pbr),
            format!("{:04X}", console.cpu.pc),
            format!("{:02X}", opcode),
            data.name.to_string(),
            format_address_mode(
                data.addr_mode,
                &[
                    c.read_byte(addr.wrapping_add(1)),
                    c.read_byte(addr.wrapping_add(2)),
                    c.read_byte(addr.wrapping_add(3)),
                ],
                data.bytes,
            ),
        ]
        .into_iter()
        .map(|t| (t, color))
        .collect()
    }
    fn next_instructions(&self) -> impl Into<Element<Message>> {
        let mut c = self.console.clone();
        let future_iter = std::iter::from_fn(move || {
            c.advance_instructions(1);
            Some(c.clone())
        });
        let it = self
            .previous_states
            .iter()
            .rev()
            .take(20)
            .map(|c| self.console_table_row(&c, Some(color!(0xAAAAAA))))
            .rev()
            .chain([self.console_table_row(&self.console, Some(color!(0xFFFFFF)))].into_iter())
            .chain(
                future_iter
                    .take(10)
                    .map(move |c| self.console_table_row(&c, Some(COLORS[2]))),
            );
        text_table(
            ["PBR", "PC", "OP", "", "       "]
                .into_iter()
                .map(|r| (r.to_string(), Some(COLORS[0])))
                .chain(it.flatten()),
            5,
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::NewFrame()),
            // event::listen().map(Message::OnEvent),
        ])
    }
    pub fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}
