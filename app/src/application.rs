use std::collections::VecDeque;

use iced::{
    Alignment::{self, Center},
    Color, Element, Event,
    Length::{self, FillPortion},
    Renderer, Subscription, Theme,
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

pub fn background_table(background: &Background, depth: usize) -> Element<'_, Message> {
    macro_rules! ppu_table_val {
        ($label: expr, $field: ident) => {
            ppu_table_val!($label, $field, "{}")
        };
        ($label: expr, $field: ident, $format_str: expr) => {
            table_row!($label, background.$field, $format_str)
        };
    }
    vertical_table(
        vec![
            ppu_table_val!("Main screen enabled", main_screen_enable),
            ppu_table_val!("Sub screen enabled", sub_screen_enable),
            ppu_table_val!("Tile size", tile_size),
            ppu_table_val!("Mosaic", mosaic),
            ppu_table_val!("Tilemap Address (Byte)", tilemap_addr, hex_fmt!()),
            table_row!(
                "Tilemap Address (Word)",
                2 * background.tilemap_addr,
                hex_fmt!()
            ),
            ppu_table_val!("CHR Address", chr_addr, hex_fmt!()),
            ppu_table_val!("H offset", h_off, hex_fmt!()),
            ppu_table_val!("V offset", v_off, hex_fmt!()),
            ppu_table_val!("# H Tilemaps", num_horz_tilemaps),
            ppu_table_val!("# V Tilemaps", num_vert_tilemaps),
        ],
        150.0,
        depth,
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
    .width(Length::Fill)
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
                scrollable(column![self.cpu_data(), self.ppu_data(), self.dma_data()])
                    .width(Length::FillPortion(25)),
                column![
                    Image::new(Handle::from_rgba(256, 240, data.as_flattened().to_vec()))
                        .height(Length::Fill)
                        .width(Length::FillPortion(50))
                        .content_fit(iced::ContentFit::Contain)
                        .filter_method(FilterMethod::Nearest),
                    row![
                        button("|<").on_press(Message::Revert),
                        button(if self.is_paused { " >" } else { "||" })
                            .on_press(Message::ChangePaused(!self.is_paused)),
                        button(">|").on_press(Message::AdvanceInstructions(1)),
                        button("To BRK")
                            .on_press(Message::AdvanceUntilOpcode(wdc65816::opcodes::ADC_I))
                    ],
                ],
                self.next_instructions()
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
    fn ppu_data(&self) -> Column<'_, Message> {
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
            text("Backgrounds:"),
            vertical_table(
                self.console
                    .ppu
                    .backgrounds
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (i.to_string(), background_table(b, 2)))
                    .collect(),
                20.0,
                1,
            )
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
    fn state_row(&self, console: &Console, color: Color) -> Row<Message> {
        let addr = console.cpu.pbr as usize * 0x10000 + console.cpu.pc as usize;
        let opcode = console.cartridge.read_byte(addr);
        let data = opcode_data(
            opcode,
            console.cpu.p.a_is_16bit(),
            console.cpu.p.xy_is_16bit(),
        );
        let c = &console.cartridge;
        Row::with_children(
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
            .map(|t| text(t).width(Length::Shrink).color(color).into()),
        )
        .spacing(10)
    }
    fn next_instructions(&self) -> Scrollable<Message> {
        let mut c = self.console.clone();
        let future_iterator = std::iter::from_fn(move || {
            c.advance_instructions(1);
            Some(self.state_row(&c, color!(0xFFAAAA)))
        });
        scrollable(iced::widget::keyed::Column::with_children(
            // Add the header row first
            [Row::with_children(
                ["PB ", "PC", "OP", "   ", ""]
                    .into_iter()
                    .map(|s| text(s).width(Length::Shrink).into()),
            )]
            .into_iter()
            // Add previous states
            .chain(
                self.previous_states
                    .iter()
                    .rev()
                    .take(10)
                    .rev()
                    .map(|c| self.state_row(c, color!(0xAAAAFF))),
            )
            // Current state
            .chain([self.state_row(&self.console, color!(0xFFFFFF))].into_iter())
            // Add the upcoming (future) instructions
            .chain(future_iterator.take(10))
            .enumerate()
            .map(|(i, r)| (i, r.into())),
        ))
        .width(Length::FillPortion(25))
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
