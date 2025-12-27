use std::fmt::UpperHex;

use iced::{
    Alignment, Color, Element, Length,
    widget::{
        Column, Row, Scrollable, column, container, row,
        scrollable::{Direction, Scrollbar},
        text,
    },
};

use crate::program::Message;

/// TODO: Generalize to not require Element
pub fn ram<T: UpperHex + Copy>(
    ram: &[T],
    page: usize,
    label_color: Color,
    byte_color: Color,
    zero_color: Color,
    addr_offset: usize,
) -> impl Into<Element<Message>>
where
    u32: From<T>,
{
    // How many bytes to show on each line
    let bytes_per_line = 0x20;
    // How many lines to show on a single page
    let num_lines = 8;
    Column::with_children(
        [Row::with_children((0..(bytes_per_line + 1)).map(|i| {
            text(if i == 0 {
                "ADDR".to_string()
            } else {
                format!("+{:02X}", i - 1)
            })
            .color(label_color)
            .align_x(if i == 0 {
                Alignment::Start
            } else {
                Alignment::End
            })
            .width(Length::Fill)
            .into()
        }))
        .into()]
        .into_iter()
        .chain((0..num_lines).into_iter().map(|y| {
            Row::with_children(
                // Add the header
                [text(format!(
                    "{:04X}",
                    addr_offset + bytes_per_line * y + bytes_per_line * num_lines * page
                ))
                .color(label_color)
                .width(Length::Fill)
                .align_x(Alignment::Start)
                .into()]
                .into_iter()
                .chain((0..bytes_per_line).into_iter().map(|x| {
                    let index = num_lines * bytes_per_line * page + bytes_per_line * y + x;
                    let color = if index >= ram.len() || u32::from(ram[index].into()) == 0 {
                        zero_color
                    } else {
                        byte_color
                    };
                    text(if index < ram.len() {
                        format!("{:02X}", ram[index])
                    } else {
                        "NA".to_string()
                    })
                    .color(color)
                    .width(Length::Fill)
                    .align_x(Alignment::End)
                    .into()
                })),
            )
            .height(Length::Fill)
            .into()
        })),
    )
}
