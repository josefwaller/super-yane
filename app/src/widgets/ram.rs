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

pub fn ram(
    ram: &[u8],
    offset: usize,
    label_color: Color,
    byte_color: Color,
    zero_color: Color,
) -> impl Into<Element<Message>> {
    let bytes_per_line = 0x20;
    let num_lines = 30;
    column![
        Row::with_children((0..(bytes_per_line + 1)).into_iter().map(|i| {
            if i == 0 {
                text("     +").into()
            } else {
                text(format!("{:02X}", i - 1)).color(label_color).into()
            }
        }))
        .spacing(10),
        Scrollable::new(Column::with_children(
            ram.chunks(bytes_per_line).enumerate().map(|(i, line)| {
                if (i + 1) * 12 < offset || (i + 1) * 12 > offset + num_lines * 12 {
                    (i, Row::new().height(12).into())
                } else {
                    (
                        i,
                        Row::with_children(
                            [text(format!("0x{:04X}", i * bytes_per_line))
                                .color(label_color)
                                .into()]
                            .into_iter()
                            .chain(line.iter().map(|l| {
                                text(format!("{:02X}", l))
                                    .color(if *l == 0 { zero_color } else { byte_color })
                                    .into()
                            })),
                        )
                        .spacing(10)
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
