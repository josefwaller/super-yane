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
    addr_offset: usize,
) -> impl Into<Element<Message>> {
    let bytes_per_line = 0x20;
    let num_lines = 30;
    // +2 for the '0x' prefix
    let addr_column_len = 2 + ((addr_offset + ram.len()).ilog2() as usize / 4 + 1);
    column![
        Row::with_children((0..(bytes_per_line + 1)).into_iter().map(|i| {
            if i == 0 {
                text(format!("{:width$}", "", width = addr_column_len)).into()
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
                            [text(format!(
                                "0x{:0width$X}",
                                i * bytes_per_line + addr_offset,
                                width = addr_column_len - 2
                            ))
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
