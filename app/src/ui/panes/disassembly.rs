use egui::{RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    disassembler::{Disassembler, Instruction},
    emulation::Emulation,
    ui::colors::{COLOR_ORANGE, LIGHT_BLUE_SECONDARY, RED_PRIMARY},
};

pub fn disassembly<I: Instruction>(ui: &mut Ui, emu: &Emulation, dis: &Disassembler<I>) {
    let mut table = TableBuilder::new(ui)
        .resizable(false)
        .columns(Column::auto(), 3)
        .column(Column::remainder());

    // Scroll to current instruction
    let index = {
        let inst = I::current_instruction(&emu.console);
        dis.instructions().keys().position(|k| *k == inst.key())
    };
    if let Some(i) = index {
        // Compute the scroll offset
        table = table.scroll_to_row(i, None);
    }
    table
        .header(12.0, |mut row| {
            row.col(|ui| {
                ui.label("  ");
            });
            row.col(|ui| {
                ui.label("PC");
            });
            row.col(|ui| {
                ui.label("OPCODE");
            });
            row.col(|ui| {
                ui.label("OPERAND(S)");
            });
        })
        .body(|body| {
            body.rows(20.0, dis.instructions().len(), |mut row| {
                let row_index = row.index();
                if let Some(inst) = dis.lines().nth(row.index()) {
                    row.col(|ui| {
                        ui.label(
                            RichText::new(if index == Some(row_index) { "->" } else { "" })
                                .color(RED_PRIMARY),
                        );
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format!("{:06X}", inst.pc)).color(COLOR_ORANGE));
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(format!("{}", inst.instruction.opcode_name()))
                                .color(LIGHT_BLUE_SECONDARY),
                        );
                    });

                    row.col(|ui| {
                        ui.label(
                            RichText::new(format!("{}", inst.instruction.operands()))
                                .color(LIGHT_BLUE_SECONDARY),
                        );
                    });
                }
            });
        });
}
