use std::collections::BTreeMap;

use log::*;
use super_yane::Console;
use wdc65816::opcodes::*;

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
static ASCII_LOWER: [char; 16] = [
    'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's', 't', 'u', 'w', 'x', 'y', 'z',
];

fn get_label_name(prefix: &str, n: usize) -> String {
    let mut chars = Vec::new();
    (0..6).for_each(|i| chars.push(ASCII_LOWER[(n >> (4 * i)) & 0xF]));
    // debug!("{} {:?}", n, chars);
    format!("{}{}", prefix, chars.into_iter().collect::<String>())
}

impl Disassembler {
    pub fn add_current_instruction(&mut self, console: &Console) {
        // Add the instruction if it is not already added
        if !self.instructions.contains_key(&console.pc()) {
            let inst = Instruction::from_console(&console);
            self.instructions
                .insert(console.cartridge().transform_address(console.pc()), inst);
            // Relative address
            if [BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS, BRA, BRL].contains(&inst.data().code) {
                // Add a label (+2 to account for the PC incrementing during execution)
                let addr = (console.pc() as isize
                    + i8::from_le_bytes([inst.operands()[0]]) as isize)
                    as usize
                    + 2;
                self.labels.insert(
                    console.cartridge().transform_address(addr),
                    Label::Location(get_label_name("", addr)),
                );
            }
        }
    }
    pub fn add_entrypoint(&mut self, console: &Console) {
        self.labels.insert(
            console.cartridge().transform_address(console.pc()),
            Label::EntryPoint,
        );
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

    pub fn labels(&self) -> &BTreeMap<usize, Label> {
        &self.labels
    }
}
