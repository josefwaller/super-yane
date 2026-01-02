use std::collections::BTreeMap;

use super_yane::Console;

use crate::disassembler::{Instruction, Label};

use derive_new::new;

/// Contains all the information required to disassemble the machine code into ASM
#[derive(new)]
pub struct Disassembler {
    /// The instructions in the disassembly
    #[new(value = "BTreeMap::new()")]
    instructions: BTreeMap<usize, Instruction>,
    /// The labels (i.e. locations that are jumped/branched to)
    #[new(value = "BTreeMap::new()")]
    labels: BTreeMap<usize, Label>,
}

impl Disassembler {
    pub fn add_current_instruction(&mut self, console: &Console) {
        // Add the instruction if it is not already added
        if !self.instructions.contains_key(&console.pc()) {
            self.instructions.insert(
                console.cartridge().transform_address(console.pc()),
                Instruction::from_console(&console),
            );
        }
    }
    // Merge all of the values of the other disassembler into this one.
    // This will remove all of the values out of other
    pub fn merge(&mut self, other: &mut Disassembler) {
        self.instructions.append(&mut other.instructions);
        self.labels.append(&mut other.labels);
    }

    pub fn instructions(&self) -> &BTreeMap<usize, Instruction> {
        &self.instructions
    }
}
