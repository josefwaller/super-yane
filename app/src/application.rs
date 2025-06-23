use iced::{
    Alignment::{self, Center},
    Color, Element, Event, Length, Renderer, Subscription, Theme,
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

#[derive(Debug, Clone)]
pub enum Message {
    NewFrame(),
    OnEvent(Event),
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
            Message::OnEvent(e) => {}
            Message::NewFrame() => self.console.advance_instructions(100),
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
                    row![button("||"), button(">"), button(">|")],
                ],
                container(text("Previous instructions")).width(Length::Shrink)
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
            text(format!("{}", self.vram_offset)),
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
    // fn vram(&self) -> Scrollable<'_, Message> {
    // let bytes_per_line = 0x20;
    // let number_lines = 20;
    // use iced::widget::keyed::Column;
    // scrollable(Column::with_children(
    //     self.console
    //         .ppu
    //         .vram
    //         .chunks(bytes_per_line)
    //         .enumerate()
    //         .map(|(i, line)| {
    //             if (i + 1) * 12 < self.vram_offset
    //                 || (i + 1) * 12 > self.vram_offset + number_lines * 12
    //             {
    //                 (i, horizontal_space().height(12).into())
    //             } else {
    //                 (
    //                     i,
    //                     row![
    //                         text(format!("0x{:04X}", i * bytes_per_line))
    //                             .color(color(TABLE_COLORS[0])),
    //                         text(
    //                             line.iter()
    //                                 .fold(String::new(), |acc, e| format!("{} {:02X}", acc, e)),
    //                         )
    //                         .wrapping(text::Wrapping::None)
    //                     ]
    //                     .height(Length::Fixed(16.0))
    //                     .into(),
    //                 )
    //             }
    //         }),
    // ))
    // .on_scroll(|v| Message::ChangeVramPage(v.absolute_offset().y as usize))
    // }

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
