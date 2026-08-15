use crate::{
    emulation::Emulation,
    ui::{
        colors::PINK_PRIMARY,
        widgets::key_value_tree::{KvNode, value_tree},
    },
};

pub fn apu_data(ui: &mut egui::Ui, emu: &Emulation) {
    let a = &emu.console.apu().core;
    value_tree(
        ui,
        &KvNode::with_children(
            "SPC700",
            "",
            &[
                KvNode::new("PC", format!("{:04X}", a.pc)),
                KvNode::new("A", format!("{:02X}", a.a)),
                KvNode::new("X", format!("{:02X}", a.x)),
                KvNode::new("Y", format!("{:02X}", a.y)),
                KvNode::new("SP", format!("{:02X}", a.sp)),
                KvNode::with_children(
                    "P",
                    format!("{:02X}", a.psw.to_byte()),
                    &[
                        KvNode::new("n", format!("{}", a.psw.n)),
                        KvNode::new("v", format!("{}", a.psw.v)),
                        KvNode::new("p", format!("{}", a.psw.p)),
                        KvNode::new("b", format!("{}", a.psw.b)),
                        KvNode::new("h", format!("{}", a.psw.h)),
                        KvNode::new("i", format!("{}", a.psw.i)),
                        KvNode::new("z", format!("{}", a.psw.z)),
                        KvNode::new("c", format!("{}", a.psw.c)),
                    ],
                ),
            ],
        ),
        0,
        &[PINK_PRIMARY, PINK_PRIMARY],
    );
}
