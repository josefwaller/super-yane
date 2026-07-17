use egui::{RichText, Ui, hex_color};

use crate::{
    disassembler::{CpuInstruction, Instruction},
    emulation::Emulation,
};

pub fn cpu_disassembly(ui: &mut Ui, emu: &Emulation) {
    let mut scroll = egui::ScrollArea::vertical().hscroll(emu.is_paused);
    let pc = emu.console.cartridge().transform_address(emu.console.pc());
    // Get height of each row (since they're just text, it's just hte text height)
    let height = ui.text_style_height(&egui::TextStyle::Body);
    if !emu.is_paused {
        // Scroll to current instruction
        let index = {
            let inst = CpuInstruction::current_instruction(&emu.console);
            emu.cpu_dis
                .instructions()
                .keys()
                .position(|k| *k == inst.key())
        };
        if let Some(i) = index {
            // Compute the scroll offset
            scroll = scroll
                .vertical_scroll_offset((height + ui.spacing().item_spacing.y) * i as f32)
                .animated(false);
        }
    }
    scroll.show_rows(
        ui,
        height,
        emu.cpu_dis.instructions().len(),
        |ui, row_range| {
            for index in row_range {
                let line = emu.cpu_dis.lines().nth(index);
                match line {
                    None => {}
                    Some(l) => {
                        let is_current_inst = if l.pc == pc { true } else { false };
                        ui.columns(4, |cols| {
                            if is_current_inst {
                                cols[0].label(RichText::new("->").color(hex_color!("FF0000")));
                            }
                            if let Some(label) = l.label {
                                cols[1].label(label.to_string());
                            }
                            cols[2].label(format!("{:06X}", l.pc));
                            cols[3].label(RichText::new(
                                l.instruction.to_string(emu.cpu_dis.labels()),
                            ));
                        })
                    }
                }
            }
        },
    );
}
