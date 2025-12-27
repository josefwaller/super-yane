use crate::program::{COLORS, Message};
use iced::{
    Element, Length,
    widget::{row, text},
};

/// TODO: Remove
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
    .into()
}
