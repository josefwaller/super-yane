use super_yane::Console;

use crate::ui::{
    colors::PINK_PRIMARY,
    vertical_table::{Row, vertical_table},
};

pub fn cartridge_data(ui: &mut egui::Ui, console: &Console) {
    let c = console.cartridge();
    vertical_table(
        ui,
        &[
            Row::new("Title", c.title.clone()),
            Row::new("Coprocessor", format!("{:?}", c.coprocessor)),
            Row::new("Developer ID", format!("{:?}", c.developer_id)),
            Row::new("Country Code", format!("{:?}", c.country_code)),
            Row::new("ROM Version", format!("{:?}", c.rom_version)),
        ],
        PINK_PRIMARY,
        ui.unique_id(),
    );
}
