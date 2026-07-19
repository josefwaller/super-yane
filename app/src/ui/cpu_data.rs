use egui::{Color32, RichText, hex_color};
use slint::Color;
use super_yane::Console;

use crate::ui::{
    colors::{LIGHT_BLUE_PRIMARY, LIGHT_BLUE_SECONDARY, RED_PRIMARY},
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
        Row::new("C", format!("{:02X}", c.c())),
        Row::new("B", format!("{:02X}", c.b)).indent(1),
        Row::new("A", format!("{:02X}", c.a)).indent(1),
        Row::new("X", format!("{:02X}", c.x())),
        Row::new("Xh", format!("{:02X}", c.xh)).indent(1),
        Row::new("Xl", format!("{:02X}", c.xl)).indent(1),
        Row::new("Y", format!("{:02X}", c.y())),
        Row::new("Yh", format!("{:02X}", c.yh)).indent(1),
        Row::new("Yl", format!("{:02X}", c.yl)).indent(1),
        Row::new("D", format!("{:02X}", c.dr())),
        Row::new("Dh", format!("{:02X}", c.dh)).indent(1),
        Row::new("Dl", format!("{:02X}", c.dl)).indent(1),
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
    vertical_table(ui, row_data, LIGHT_BLUE_PRIMARY, "CPU".to_string());
}
