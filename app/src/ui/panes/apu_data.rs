use crate::{
    emulation::Emulation,
    ui::{
        colors::PINK_PRIMARY,
        widgets::key_value_tree::{KvNode, value_tree},
    },
};

pub fn apu_data(ui: &mut egui::Ui, emu: &Emulation) {
    let a = &emu.console.apu().core;
    let timer_vals: [[KvNode; 2]; 3] = core::array::from_fn(|i| {
        let t = &emu.console.apu().timers()[i];
        [
            KvNode::new("Counter", format!("{:02X}", t.counter)),
            KvNode::new("Target", format!("{:02X}", t.target)),
        ]
    });
    value_tree(
        ui,
        &KvNode::with_children(
            "APU",
            "",
            &[
                KvNode::with_children(
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
                KvNode::with_children(
                    "APU to CPU",
                    "",
                    &[0, 1, 2, 3].map(|i| {
                        KvNode::new(
                            format!("{:02X}", 0xF4 + i),
                            format!("{:02X}", emu.console.apu_to_cpu_reg()[i]),
                        )
                    }),
                ),
                KvNode::with_children(
                    "CPU to APU",
                    "",
                    &[0, 1, 2, 3].map(|i| {
                        KvNode::new(
                            format!("{:04X}", 0x2140 + i),
                            format!("{:02X}", emu.console.cpu_to_apu_reg()[i]),
                        )
                    }),
                ),
                KvNode::with_children(
                    "Timers",
                    "",
                    &[0, 1, 2].map(|i| KvNode::with_children("Timer ", "", &timer_vals[i])),
                ),
            ],
        ),
        0,
        &[PINK_PRIMARY, PINK_PRIMARY],
    );
}
