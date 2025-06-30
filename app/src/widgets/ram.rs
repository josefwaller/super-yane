use iced::{
    Color, Element, Length,
    widget::{
        Row, Scrollable, column, container, horizontal_space,
        keyed::Column,
        row,
        scrollable::{Direction, Scrollbar},
        text,
    },
};

use crate::application::Message;

pub fn ram(ram: &[u8], offset: usize, label_color: Color) -> impl Into<Element<Message>> {
    let bytes_per_line = 0x20;
    let num_lines = 30;
    column![
        row![
            text(
                (0..(bytes_per_line))
                    .into_iter()
                    .fold("     +".to_string(), |acc, e| format!("{} {:02X}", acc, e))
            )
            .color(label_color)
        ],
        Scrollable::new(Column::with_children(
            ram.chunks(bytes_per_line).enumerate().map(|(i, line)| {
                if (i + 1) * 12 < offset || (i + 1) * 12 > offset + num_lines * 12 {
                    (i, Row::new().height(12).into())
                } else {
                    (
                        i,
                        row![
                            text(format!("0x{:04X}", i * bytes_per_line)).color(label_color),
                            text(
                                line.iter()
                                    .fold(String::new(), |acc, e| format!("{} {:02X}", acc, e)),
                            )
                            .wrapping(text::Wrapping::None)
                        ]
                        .height(Length::Fixed(16.0))
                        .into(),
                    )
                }
            }),
        ))
        .on_scroll(|v| Message::ChangeVramPage(v.absolute_offset().y as usize))
        .spacing(10)
    ]
}
