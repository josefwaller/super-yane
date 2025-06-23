use iced::{
    Color, Length,
    widget::{Scrollable, horizontal_space, keyed::Column, row, scrollable, text},
};

use crate::application::Message;

pub fn ram(ram: &[u8], offset: usize, label_color: Color) -> Scrollable<'_, Message> {
    let bytes_per_line = 0x20;
    let num_lines = 30;
    scrollable(Column::with_children(
        ram.chunks(bytes_per_line).enumerate().map(|(i, line)| {
            if (i + 1) * 12 < offset || (i + 1) * 12 > offset + num_lines * 12 {
                (i, horizontal_space().height(12).into())
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
}
