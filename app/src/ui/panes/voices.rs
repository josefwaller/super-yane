use egui::Ui;

use crate::{
    emulation::Emulation,
    ui::{
        colors::PINK_PRIMARY,
        widgets::key_value_tree::{KvNode, enabled_flag, value_tree},
    },
};

pub fn voices(ui: &mut Ui, emu: &Emulation) {
    ui.vertical(|ui| {
        for (i, v) in emu.console.apu().dsp().voices.iter().enumerate() {
            value_tree(
                ui,
                &KvNode::with_children(
                    format!("Voice {}", i),
                    "",
                    &[
                        KvNode::new("Sample Pitch", v.sample_pitch),
                        KvNode::new("Pitch Modulation", enabled_flag(v.pitch_mod_enabled)),
                        KvNode::new("Is Releasing", v.is_releasing),
                        KvNode::new(
                            "Volume",
                            format!("L: {:02X} R: {:02X}", v.volume[0], v.volume[1]),
                        ),
                        KvNode::new("Sample Source", v.sample_src),
                        KvNode::with_children(
                            "ADSR",
                            enabled_flag(v.adsr_enabled),
                            &[
                                KvNode::new("Stage", format!("{:?}", v.adsr_stage)),
                                KvNode::new("Attack Rate", v.attack_rate),
                                KvNode::new("Decay Rate", v.decay_rate),
                                KvNode::new("Sustain Rate", v.sustain_rate),
                                KvNode::new("Sustain Level", format!("{:X}", v.sustain_level)),
                            ],
                        ),
                        KvNode::with_children(
                            "Gain",
                            enabled_flag(!v.adsr_enabled),
                            &[
                                KvNode::new("Mode", format!("{:?}", v.gain_mode)),
                                KvNode::new("Rate", v.gain_rate),
                            ],
                        ),
                        KvNode::new("Echo", enabled_flag(v.echo_enabled)),
                        KvNode::new("End Flag", v.end_flag),
                        KvNode::new("Envelope", v.envelope),
                        KvNode::new("Noise", enabled_flag(v.noise_enabled)),
                    ],
                ),
                0,
                &[PINK_PRIMARY],
            );
        }
    });
}
