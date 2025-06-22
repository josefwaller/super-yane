use iced::{
    Alignment::Center,
    Color, Element, Event, Length, Subscription, Theme,
    alignment::Horizontal,
    event,
    widget::{
        Column, Row, button, column, container, horizontal_space,
        image::{Handle, Image},
        keyed_column, row, scrollable,
        text::IntoFragment,
    },
    window,
};

use iced::widget::text;

use super_yane::{Console, ppu::Background};

macro_rules! hex_fmt {
    () => {
        "0x{:04X}"
    };
}

fn color(hex: u32) -> Color {
    Color::from_rgb8(
        (hex >> 16) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
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
            ppu_table_val!("# H Tilemaps", num_horz_tilemaps),
            ppu_table_val!("# V Tilemaps", num_vert_tilemaps),
        ],
        150.0,
        depth,
    )
}
const TABLE_COLORS: [u32; 4] = [0x98c2d4, 0xd49e98, 0x91c29c, 0xc2af91];
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
                    .color(color(TABLE_COLORS[depth])),
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
}

pub struct Application {
    console: Console,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            console: Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc")),
        }
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::OnEvent(e) => {}
            Message::NewFrame() => self.console.advance_instructions(100),
            _ => {}
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
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
                container(button("Pause")).width(Length::Shrink),
                Image::new(Handle::from_rgba(256, 240, data.as_flattened().to_vec()))
                    .height(Length::Fill)
                    .width(Length::FillPortion(50))
                    .content_fit(iced::ContentFit::Contain),
                container(text("Previous instructions")).width(Length::Shrink)
            ]
            .padding(10)
            .spacing(10)
            .height(Length::Fill),
            row![self.ppu_data(), text("Thing Two"), text("thing three")]
                .spacing(10)
                .height(Length::Fixed(200.0))
        ]
        .padding(10)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }

    fn ppu_data(&self) -> Element<'_, Message> {
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
            (
                "Backgrounds",
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
                ),
            ),
        ];
        scrollable(vertical_table(values, 150.0, 0)).into()
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
