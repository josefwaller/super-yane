use iced::{
    Color, Element, color,
    widget::{Row, Text, keyed::Column as KeyedColumn},
};

#[macro_export]
macro_rules! cell {
    ($label: expr, $val: ident, $format_str: expr, $color: ident) => {
        [
            ($label.to_string(), color!($color)),
            (format!($format_str, $val)),
        ]
    };
}

/// A simple table that renders text-only rows
pub fn text_table<'a, Message: 'a>(
    rows_iter: impl Iterator<Item = (String, Option<Color>)>,
    num_columns: usize,
) -> impl Into<Element<'a, Message>> {
    let rows: Vec<(String, Option<Color>)> = rows_iter.collect();
    let column_widths =
        rows.iter()
            .enumerate()
            .fold(vec![0; num_columns], |mut acc, (i, (r, _))| {
                if acc[i % num_columns] < r.len() {
                    acc[i % num_columns] = r.len();
                }
                acc
            });
    KeyedColumn::with_children(
        rows.chunks(num_columns)
            .enumerate()
            .map(|(i, row)| {
                (
                    i,
                    Row::with_children(row.iter().enumerate().map(|(j, (contents, color))| {
                        Text::new(format!("{:<width$}", contents, width = column_widths[j]))
                            .color_maybe(*color)
                            .into()
                    }))
                    .spacing(10)
                    .into(),
                )
            })
            .into_iter(),
    )
}
