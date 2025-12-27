use crate::program::{COLORS, Message};
use iced::Element;
use super_yane::Background;

use crate::widgets::text_table;

pub fn background_table(background: &Background) -> impl Into<Element<'_, Message>> {
    let b = background;
    let rows = vec![
        ("Main screen enabled", b.main_screen_enable.into()),
        ("Sub screen enabled", b.sub_screen_enable.into()),
        ("Tile size", b.tile_size),
        ("Mosaic", b.mosaic.into()),
        ("Tilemap Address (Byte)", 2 * b.tilemap_addr as u32),
        ("Tilemap Address (Word)", b.tilemap_addr as u32),
        ("CHR Address", b.chr_addr as u32),
        ("H offset", b.h_off),
        ("V offset", b.v_off),
        ("# H Tilemaps", b.num_horz_tilemaps),
        ("# V Tilemaps", b.num_vert_tilemaps),
        ("Windows enabled main", b.windows_enabled_main.into()),
        ("Windows enabled sub", b.windows_enabled_sub.into()),
    ];
    text_table(
        rows.into_iter()
            .map(|(s, v)| {
                [
                    (s.to_string(), Some(COLORS[1])),
                    (format!("{:02X}", v), None),
                ]
            })
            .flatten(),
        2,
    )
}
