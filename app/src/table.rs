use iced::{
    Alignment, Color, Element, Length, color,
    widget::{Column, Row, keyed::Column as KeyedColumn, text},
};

#[derive(Clone)]
pub struct Cell {
    contents: String,
    color: Color,
}

const DEFAULT_COLOR: Color = Color::WHITE;
const HEADER_COLOR: Color = color!(0xFFADAD);

impl<'a, E> From<Cell> for Element<'a, E> {
    fn from(value: Cell) -> Self {
        text(value.contents).color(value.color).into()
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
    values: impl IntoIterator<Item = [Cell; W]>,
) -> impl Into<Element<'a, E>> {
    let v: Vec<[Cell; W]> = values.into_iter().collect();
    Row::with_children((0..W).map(move |i| {
        Column::with_children(
            [h_cell(headers[i].clone().into())]
                .into_iter()
                .chain(v.iter().map(|row| row[i].clone()))
                .map(|e| e.into()),
        )
        .align_x(Alignment::End)
        .into()
    }))
    .spacing(10)
    .padding(10)
}
