use std::collections::BTreeMap;

use itertools::Itertools;
use log::*;
use super_yane::Console;
use wdc65816::opcodes::*;

use crate::{
    // DisassemblyLine,
    disassembler::{Instruction, Label},
};

use derive_new::new;

#[derive(new)]
pub struct LinesIterator<'a, I>
where
    I: Instruction,
{
    instruction_index: usize,
    labels_index: usize,
    instructions: &'a BTreeMap<usize, I>,
    /// Labels only contains the reset/interrupt vectors
    labels: &'a BTreeMap<usize, Label>,
}

pub struct Line<I>
where
    I: Instruction,
{
    pub pc: usize,
    pub label: Option<Label>,
    pub instruction: I,
}

impl<I> Line<I>
where
    I: Instruction,
{
    pub fn new(pc: usize, instruction: I, labels: &BTreeMap<usize, Label>) -> Self {
        Line {
            pc: pc,
            label: labels.get(&pc).map(|l| l.to_owned()),
            instruction,
        }
    }
}

impl<'a, I> Iterator for LinesIterator<'a, I>
where
    I: Instruction,
{
    type Item = Line<I>;
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.instructions
            .iter()
            .nth(n)
            .map(|(pc, i)| Line::new(*pc, i.clone(), self.labels))
    }
    fn next(&mut self) -> Option<Self::Item> {
        let v = self
            .instructions
            .iter()
            .nth(self.instruction_index)
            .map(|(pc, i)| Line::new(*pc, i.clone(), self.labels));
        self.instruction_index += 1;
        v
    }
}

/// Contains all the information required to disassemble the machine code into ASM
#[derive(Clone)]
pub struct Disassembler<I>
where
    I: Instruction,
{
    /// The instructions in the disassembly
    instructions: BTreeMap<usize, I>,
    /// The labels (i.e. locations that are jumped/branched to)
    labels: BTreeMap<usize, Label>,
}

impl<I: Instruction> Disassembler<I> {
    pub fn add_current_instruction(&mut self, console: &Console) {
        // Add the instruction if it is not already added
        let inst = I::current_instruction(&console);
        let key = inst.key();
        self.instructions.insert(key, inst.clone());
    }
    pub fn new() -> Disassembler<I> {
        Disassembler {
            instructions: BTreeMap::new(),
            labels: BTreeMap::new(),
        }
    }
    pub fn add_native_vectors(&mut self, console: &Console) {
        macro_rules! vector {
            ($addr: expr) => {
                console
                    .cartridge()
                    .transform_address(u16::from_le_bytes(core::array::from_fn(|i| {
                        console.cartridge().read_byte($addr + i)
                    })) as usize)
            };
        }
        self.labels.append(&mut BTreeMap::from_iter(
            [
                (vector!(0x00FFFC), Label::Reset),
                (vector!(0x00FFFE), Label::IrqEmu),
                (vector!(0x00FFEE), Label::IrqNative),
                (vector!(0x00FFFA), Label::NmiEmu),
                (vector!(0x00FFEA), Label::NmiNative),
            ]
            .into_iter()
            .unique_by(|(addr, _)| *addr),
        ));
    }
    // Merge all of the values of the other disassembler into this one.
    // This will remove all of the values out of other
    pub fn consume(&mut self, other: &mut Disassembler<I>) {
        self.instructions.append(&mut other.instructions);
        self.labels.append(&mut other.labels);
    }

    pub fn instructions(&self) -> &BTreeMap<usize, I> {
        &self.instructions
    }

    /// Iterator over the lines in the disassembly so far
    pub fn lines(&self) -> impl Iterator<Item = Line<I>> {
        return LinesIterator::new(0, 0, &self.instructions, &self.labels);
    }
    pub fn labels(&self) -> &BTreeMap<usize, Label> {
        &self.labels
    }
}
