use std::collections::BTreeMap;

use derive_new::new;
use super_yane::Console;

use crate::instruction_snapshot::InstructionSnapshot;

#[derive(new)]
pub struct Profiler {
    #[new(value = "[0; 0x100]")]
    pub opcode_cycles: [u64; 0x100],
    #[new(value = "BTreeMap::new()")]
    pub pc_count: BTreeMap<usize, (u32, InstructionSnapshot)>,
}

impl Profiler {
    pub fn add_current_state(&mut self, console: &Console, last_master_cycles: u64) {
        // Add current opcode
        self.opcode_cycles[console.opcode() as usize] +=
            console.total_master_clocks() - last_master_cycles;
        // Add current PC
        let count = self
            .pc_count
            .get(&console.pc())
            .map(|(count, _)| count)
            .unwrap_or(&0);
        self.pc_count.insert(
            console.pc(),
            (count + 1, InstructionSnapshot::from(console)),
        );
    }
    /// Merges `other` into this [`Profiler`] and reset `other`
    pub fn consume(&mut self, other: &mut Profiler) {
        (0..0x100).for_each(|i| self.opcode_cycles[i] += other.opcode_cycles[i]);
        other.pc_count.iter().for_each(|(k, (c, inst))| {
            self.pc_count.insert(
                *k,
                (
                    self.pc_count.get(k).map(|(count, _)| count).unwrap_or(&0) + c,
                    inst.clone(),
                ),
            );
        });
        *other = Profiler::new()
    }
}
