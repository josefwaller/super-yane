use std::time::Duration;

use super_yane::{APU_CR_CLOCK_SPEED_HZ, Console, MASTER_CLOCK_SPEED_HZ};

use crate::ui::{
    colors::PINK_PRIMARY,
    widgets::key_value_tree::{KvNode, value_tree},
};

pub fn info(ui: &mut egui::Ui, console: &Console) {
    let c = console.cartridge();
    value_tree(
        ui,
        &KvNode::with_children(
            "Info",
            String::new(),
            &[
                KvNode::new("Title", c.title.clone()),
                KvNode::new("Coprocessor", format!("{:?}", c.coprocessor)),
                KvNode::new("Developer ID", format!("{:?}", c.developer_id)),
                KvNode::new("Country Code", format!("{:?}", c.country_code)),
                KvNode::new("ROM Version", format!("{:?}", c.rom_version)),
                KvNode::new(
                    "CPU clocks",
                    format!(
                        "{} ({:?})",
                        console.total_master_clocks(),
                        Duration::from_nanos(
                            *console.total_master_clocks() * 1_000_000_000 / MASTER_CLOCK_SPEED_HZ
                        )
                    ),
                ),
                KvNode::with_children(
                    "APU clock speed",
                    String::new(),
                    &[
                        KvNode::new(
                            "Total clocks",
                            format!("{}", console.apu().total_cr_clocks()),
                        ),
                        KvNode::new(
                            "Elapsed time",
                            format!(
                                "{:?}",
                                Duration::from_nanos(
                                    *console.apu().total_cr_clocks() as u64 * 1_000_000_000
                                        / APU_CR_CLOCK_SPEED_HZ
                                )
                            ),
                        ),
                        KvNode::new(
                            "Total SPC clocks",
                            format!("{}", console.apu().total_core_clocks()),
                        ),
                    ],
                ),
            ],
        ),
        0,
        &[PINK_PRIMARY],
    );
}
