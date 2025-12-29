use iced::{
    Alignment, Color, Element, Length, color,
    theme::Style,
    widget::{Grid, Row, container, keyed::Column as KeyedColumn, text},
};

#[derive(Clone)]
pub struct Cell {
    contents: String,
    color: Color,
}

const DEFAULT_COLOR: Color = Color::WHITE;
const HEADER_COLOR: Color = color!(0xFFADAD);

impl<'a, E: 'a> From<Cell> for Element<'a, E> {
    fn from(value: Cell) -> Self {
        container(text(value.contents).color(value.color))
            .center_y(Length::Shrink)
            .into()
    }
}

pub fn cell(s: impl Into<String>) -> Cell {
    Cell {
        contents: s.into(),
        color: DEFAULT_COLOR,
    }
}
pub fn h_cell(s: impl Into<String>) -> Cell {
    Cell {
        contents: s.into(),
        color: HEADER_COLOR,
    }
}

pub fn table<'a, const W: usize, E: 'a>(
    headers: [impl Into<String> + Clone; W],
    values: impl IntoIterator<Item = [Element<'a, E>; W]>,
) -> impl Into<Element<'a, E>> {
    Grid::with_children(
        [headers.map(|s| h_cell(s.into()).into())]
            .into_iter()
            .chain(values)
            .flatten(),
    )
    .columns(W)
    .spacing(10)
    .height(Length::Shrink)
}
