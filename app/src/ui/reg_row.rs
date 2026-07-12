use egui::{Color32, RichText, TextStyle, WidgetText};

/// Shared method to render a register's value in a grid view.
/// Must be caled in a Grid
pub fn reg_row(
    ui: &mut egui::Ui,
    label: impl Into<WidgetText>,
    value: impl Into<WidgetText>,
    indent: u32,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0 * indent as f32);
        ui.label(label);
    });
    ui.label(value);
    ui.end_row();
}
pub fn high_low_reg(
    ui: &mut egui::Ui,
    label: impl Into<WidgetText>,
    high_label: impl Into<WidgetText>,
    low_label: impl Into<WidgetText>,
    value: u16,
    high_value: u8,
    low_value: u8,
) {
    reg_row(ui, label, format!("{:04X}", value), 0);
    reg_row(ui, high_label, format!("{:02X}", high_value), 1);
    reg_row(ui, low_label, format!("{:02X}", low_value), 1);
}
