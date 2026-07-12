use egui::{Color32, RichText, hex_color};
use slint::Color;
use super_yane::Console;

use crate::ui::reg_row::{high_low_reg, reg_row};

pub fn cpu_data(ui: &mut egui::Ui, console: &Console) {
    egui::Grid::new("CpuData").num_columns(2).show(ui, |ui| {
        reg_row(ui, "PC", format!("{:06X}", console.pc()), 0);
        let c = console.cpu();
        high_low_reg(ui, "C", "B", "A", c.c(), c.b, c.a);
        high_low_reg(ui, "X", "Xh", "Xl", c.x(), c.xh, c.xl);
        high_low_reg(ui, "Y", "Yh", "Yl", c.y(), c.yh, c.yl);
        high_low_reg(ui, "D", "Dh", "Dl", c.dr(), c.dh, c.dl);
        reg_row(ui, "DBR", format!("{:02X}", c.dbr), 0);
        reg_row(ui, "SR", format!("{:04X}", c.s), 0);
        reg_row(ui, "P (read)", format!("{:02X}", c.p.to_byte(true)), 0);
        reg_row(ui, "P (actual)", format!("{:02X}", c.p.to_byte(false)), 0);
        reg_row(ui, "P.c", format!("{}", u8::from(c.p.c)), 1);
        reg_row(ui, "P.z", format!("{}", u8::from(c.p.z)), 1);
        reg_row(ui, "P.n", format!("{}", u8::from(c.p.n)), 1);
        reg_row(ui, "P.d", format!("{}", u8::from(c.p.d)), 1);
        reg_row(ui, "P.i", format!("{}", u8::from(c.p.i)), 1);
        reg_row(ui, "P.m", format!("{}", u8::from(c.p.m)), 1);
        reg_row(ui, "P.v", format!("{}", u8::from(c.p.v)), 1);
        reg_row(ui, "P.e", format!("{}", u8::from(c.p.e)), 1);
        reg_row(ui, "P.xb", format!("{}", u8::from(c.p.xb)), 1);
    });
}
