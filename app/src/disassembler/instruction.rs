use crate::disassembler::Label;
use std::collections::BTreeMap;
use super_yane::Console;
use wdc65816::{OpcodeData, format_address_mode, opcode_data};

#[derive(Clone, Copy)]
pub struct Instruction {
    pc: usize,
    opcode: u8,
    operands: [u8; 3],
    a: bool,
    xy: bool,
}

impl Instruction {
    pub fn from_console(value: &Console) -> Self {
        Instruction {
            pc: value.cartridge().transform_address(value.pc()),
            opcode: value.opcode(),
            operands: core::array::from_fn(|i| value.read_byte_cpu(value.pc() + i + 1)),
            a: value.cpu().p.a_is_16bit(),
            xy: value.cpu().p.xy_is_16bit(),
        }
    }
    pub fn to_string(&self, labels: &BTreeMap<usize, Label>) -> String {
        let data = opcode_data(self.opcode, self.a, self.xy);
        let operands = self
            .get_jump_addr(self.pc)
            .map(|addr| labels.get(&addr).map(|l| l.to_string()))
            .flatten()
            .unwrap_or(format_address_mode(
                data.addr_mode,
                &self.operands,
                data.bytes,
            ));
        format!("{} {}", data.name, operands)
    }
    pub fn data(&self) -> OpcodeData {
        opcode_data(self.opcode, self.a, self.xy)
    }
    pub fn operands(&self) -> &[u8] {
        &self.operands[0..(self.data().bytes as usize)]
    }
    /// Get the address this instruction will jump to (given the PC), or None
    pub fn get_jump_addr(&self, pc: usize) -> Option<usize> {
        use wdc65816::opcodes::*;
        // Relative address
        if [BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS, BRA, BRL].contains(&self.opcode) {
            // Add a label (+2 to account for the PC incrementing during execution)
            Some((pc as isize + i8::from_le_bytes([self.operands()[0]]) as isize) as usize + 2)
        } else {
            None
        }
    }
}
