use iced::{
    Alignment::{self, Center},
    Color, Element, Event,
    Length::{self, FillPortion},
    Renderer, Subscription, Theme,
    alignment::Horizontal,
    color, event,
    widget::{
        Column, Container, Row, Scrollable, Slider, button, column, container, horizontal_space,
        image::{Handle, Image},
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
            ppu_table_val!("Tilemap Address", tilemap_addr, hex_fmt!()),
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

pub const COLORS: [Color; 4] = [
    color!(0x98c2d4),
    color!(0xd49e98),
    color!(0x91c29c),
    color!(0xc2af91),
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
    // OnEvent(Event),
    ChangeVramPage(usize),
}

pub struct Application {
    console: Console,
    vram_offset: usize,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            console: Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc")),
            vram_offset: 0,
        }
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            // Message::OnEvent(e) => {}
            Message::NewFrame() => {}
            Message::AdvanceInstructions(num_instructions) => {
                self.console.advance_instructions(num_instructions)
            }
            Message::ChangeVramPage(new_vram_page) => self.vram_offset = new_vram_page,
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        debug!("{:}", self.vram_offset);
        let data: [[u8; 4]; 8] = core::array::from_fn(|i| {
            [
                (i as u8 & 0x01) * 0xFF,
                (i as u8 & 0x02) / 0x02 * 0xFF,
                (i as u8 & 0x04) / 0x04 * 0xFF,
                0xFF,
            ]
        });
        let data = self.console.ppu.screen_buffer.map(|i| data[i as usize]);
        column![
            row![
                self.ppu_data().width(Length::FillPortion(25)),
                column![
                    Image::new(Handle::from_rgba(256, 240, data.as_flattened().to_vec()))
                        .height(Length::Fill)
                        .width(Length::FillPortion(50))
                        .content_fit(iced::ContentFit::Contain),
                    row![
                        button("||"),
                        button(">"),
                        button(">|").on_press(Message::AdvanceInstructions(1))
                    ],
                ],
                self.next_instructions()
            ]
            .spacing(10)
            .height(Length::Fill),
            row![
                ram(&self.console.ppu.vram, self.vram_offset, COLORS[3])
                    .on_scroll(|v| Message::ChangeVramPage(v.absolute_offset().y as usize))
            ]
            .spacing(10)
            .height(Length::Fixed(200.0))
        ]
        .padding(10)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }

    fn ppu_data(&self) -> scrollable::Scrollable<'_, Message> {
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
        ];
        scrollable(column![
            text(format!("{}", self.console.cpu.pc)),
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
        ])
    }
    fn next_instructions(&self) -> Scrollable<Message> {
        let mut pc = self.console.cpu.pc;
        scrollable(iced::widget::keyed::Column::with_children(
            // Add the header row first
            [(
                text("PB PC  ").color(COLORS[0]),
                text("OP ").color(COLORS[0]).align_x(Horizontal::Right),
                text("Operand").color(COLORS[0]),
            )]
            .into_iter()
            // Add the upcoming (future) instructions
            .chain((0..100).map(|_| {
                let c = &self.console;
                let addr = c.cpu.pbr as usize * 0x10000 + pc as usize;
                let opcode = c.cartridge.read_byte(addr);
                let data = opcode_data(opcode, c.cpu.p.a_is_16bit(), c.cpu.p.xy_is_16bit());
                let v = (
                    text(format!("{:02X} {:04X}", c.cpu.pbr, pc)),
                    text(data.name).align_x(Horizontal::Right),
                    text(format_address_mode(
                        data.addr_mode,
                        &[
                            c.cartridge.read_byte(addr + 1),
                            c.cartridge.read_byte(addr + 2),
                            c.cartridge.read_byte(addr + 3),
                        ],
                        data.bytes,
                    )),
                );
                pc = pc.wrapping_add(data.bytes as u16);
                v
            }))
            .enumerate()
            .map(|(i, (a, b, c))| {
                (
                    i,
                    row![
                        a.width(Length::Shrink),
                        b.width(Length::Shrink),
                        c.width(Length::Fill)
                    ]
                    .spacing(10)
                    .into(),
                )
            }),
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
