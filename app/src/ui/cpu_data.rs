use egui::{Color32, RichText, hex_color};
use slint::Color;
use super_yane::Console;

use crate::ui::{
    colors::{COLOR_LIGHT_BLUE, LIGHT_BLUE_PRIMARY, RED_PRIMARY},
    vertical_table::{Row, vertical_table},
};

// fn high_low_reg
pub fn cpu_data(ui: &mut egui::Ui, console: &Console) {
    let c = &console.cpu();
    let row_data = &[
        Row::new("PC", format!("{:06X}", console.pc())),
        Row::new(
            "PC (Transformed)",
            format!(
                "{:06X}",
                console.cartridge().transform_address(console.pc())
            ),
        ),
        // high_low_regRow::new(ui, "C", "B", "A", c.c(), c.b, c.a),;
        // high_low_regRow::new(ui, "X", "Xh", "Xl", c.x(), c.xh, c.xl),;
        // high_low_regRow::new(ui, "Y", "Yh", "Yl", c.y(), c.yh, c.yl),;
        // high_low_regRow::new(ui, "D", "Dh", "Dl", c.dr(), c.dh, c.dl),;
        Row::new("DBR", format!("{:02X}", c.dbr)),
        Row::new("SR", format!("{:04X}", c.s)),
        Row::new("P (read)", format!("{:02X}", c.p.to_byte(true))),
        Row::new("P (actual)", format!("{:02X}", c.p.to_byte(false))).indent(1),
        Row::new("P.c", format!("{}", u8::from(c.p.c))).indent(1),
        Row::new("P.z", format!("{}", u8::from(c.p.z))).indent(1),
        Row::new("P.n", format!("{}", u8::from(c.p.n))).indent(1),
        Row::new("P.d", format!("{}", u8::from(c.p.d))).indent(1),
        Row::new("P.i", format!("{}", u8::from(c.p.i))).indent(1),
        Row::new("P.m", format!("{}", u8::from(c.p.m))).indent(1),
        Row::new("P.v", format!("{}", u8::from(c.p.v))).indent(1),
        Row::new("P.e", format!("{}", u8::from(c.p.e))).indent(1),
        Row::new("P.xb", format!("{}", u8::from(c.p.xb))).indent(1),
    ];
    vertical_table(ui, row_data, LIGHT_BLUE_PRIMARY);
}
