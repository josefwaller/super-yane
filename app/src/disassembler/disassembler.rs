use std::collections::BTreeMap;

use log::*;
use super_yane::Console;
use wdc65816::opcodes::*;

use crate::disassembler::{Instruction, Label};

use derive_new::new;

#[derive(new)]
pub struct LinesIterator<'a> {
    instruction_index: usize,
    labels_index: usize,
    instructions: &'a BTreeMap<usize, Instruction>,
    labels: &'a BTreeMap<usize, Label>,
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let inst = self
            .instructions
            .iter()
            .nth(self.instruction_index)
            .map(|(pc, i)| (pc, format!("{:8}{}", " ", i.to_string(self.labels))));
        let lab = self
            .labels
            .iter()
            .nth(self.labels_index)
            .map(|(pc, l)| (pc, format!("{}:", l.to_string())));
        if inst.is_some() {
            let (ipc, i) = inst.unwrap();
            Some(if lab.is_some() {
                let (lpc, l) = lab.unwrap();
                if lpc <= ipc {
                    self.labels_index += 1;
                    l
                } else {
                    self.instruction_index += 1;
                    i
                }
            } else {
                self.instruction_index += 1;
                i
            })
        } else {
            lab.map(|(_, l)| {
                self.labels_index += 1;
                l
            })
        }
    }
}

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
    format!("{}{}", prefix, chars.into_iter().collect::<String>())
}

impl Disassembler {
    pub fn add_current_instruction(&mut self, console: &Console) {
        // Add the instruction if it is not already added
        let key = console.cartridge().transform_address(console.pc());
        if !self.instructions.contains_key(&key) {
            let inst = Instruction::from_console(&console);
            self.instructions.insert(key, inst);
            if let Some(addr) = inst.get_jump_addr(console.pc()) {
                let addr = console.cartridge().transform_address(addr);
                self.labels
                    .insert(addr, Label::Location(get_label_name("", addr)));
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
    pub fn consume(&mut self, other: &mut Disassembler) {
        self.instructions.append(&mut other.instructions);
        self.labels.append(&mut other.labels);
    }

    pub fn instructions(&self) -> &BTreeMap<usize, Instruction> {
        &self.instructions
    }

    pub fn labels(&self) -> &BTreeMap<usize, Label> {
        &self.labels
    }

    /// Iterator over the lines in the disassembly so far
    pub fn lines(&self) -> impl Iterator<Item = String> {
        return LinesIterator::new(0, 0, &self.instructions, &self.labels);
    }
}
