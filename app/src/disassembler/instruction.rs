use crate::disassembler::Label;
use log::*;
use spc700::{OpcodeData as ApuOpcodeData, format_address_modes};
use std::collections::BTreeMap;
use super_yane::{Cartridge, Console, cartridge::MemoryMap};
use wdc65816::{OpcodeData, format_address_mode, opcode_data};

pub trait Instruction: Clone {
    fn key(&self) -> usize;
    fn current_instruction(console: &Console) -> Self;
    fn jump_addr(&self, pc: usize) -> Option<usize>;
    fn addr(&self) -> String;
    fn opcode_name(&self) -> String;
    fn operands(&self, labels: &BTreeMap<usize, Label>) -> String;
}

#[derive(Clone, Copy)]
pub struct CpuInstruction {
    pc: usize,
    opcode: u8,
    operands: [u8; 3],
    a: bool,
    xy: bool,
}

impl CpuInstruction {
    pub fn to_string(&self, labels: &BTreeMap<usize, Label>) -> String {
        let data = self.data();
        let operands = self.operands(labels);
        format!("{} {}", data.name, operands)
    }
    pub fn data(&self) -> OpcodeData {
        opcode_data(self.opcode, self.a, self.xy)
    }
    pub fn operand_bytes(&self) -> &[u8] {
        &self.operands[0..(self.data().bytes as usize)]
    }
}

impl Instruction for CpuInstruction {
    fn key(&self) -> usize {
        self.pc as usize
    }
    fn current_instruction(value: &Console) -> Self {
        CpuInstruction {
            pc: value.cartridge().transform_address(value.pc()),
            opcode: value.opcode(),
            operands: core::array::from_fn(|i| value.read_byte_cpu(value.pc() + i + 1)),
            a: value.cpu().p.a_is_16bit(),
            xy: value.cpu().p.xy_is_16bit(),
        }
    }
    /// Get the address this instruction will jump to (given the PC), or None
    fn jump_addr(&self, pc: usize) -> Option<usize> {
        use wdc65816::opcodes::*;
        // Relative address
        if [BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS, BRA, BRL].contains(&self.opcode) {
            // Add a label (+2 to account for the PC incrementing during execution)
            Some((pc as isize + i8::from_le_bytes([self.operand_bytes()[0]]) as isize) as usize + 2)
        // Absolute address
        } else if [JMP_A, JSR_A].contains(&self.opcode) {
            Some(u16::from_le_bytes(core::array::from_fn(|i| self.operand_bytes()[i])) as usize)
        } else if [JMP_AL, JSL].contains(&self.opcode) {
            Some(u32::from_le_bytes(core::array::from_fn(|i| {
                if i < self.operands.len() {
                    self.operands[i]
                } else {
                    0
                }
            })) as usize)
        } else {
            None
        }
    }
    fn opcode_name(&self) -> String {
        self.data().name.to_string()
    }
    fn operands(&self, labels: &BTreeMap<usize, Label>) -> String {
        let data = self.data();
        self.jump_addr(self.pc)
            .map(|addr| {
                labels
                    .get(&(MemoryMap::LoRom.transform_address(addr)))
                    .map(|l| l.to_string())
            })
            .flatten()
            .unwrap_or(format_address_mode(
                data.addr_mode,
                &self.operands,
                data.bytes,
            ))
    }
    fn addr(&self) -> String {
        format!("{:06X}", self.pc)
    }
}

#[derive(Copy, Clone)]
pub struct ApuInstruction {
    pc: u16,
    opcode: u8,
    operands: [u8; 3],
}

impl Instruction for ApuInstruction {
    fn key(&self) -> usize {
        self.pc as usize
    }
    fn current_instruction(console: &Console) -> Self {
        let p = &console.apu().core;
        ApuInstruction {
            pc: p.pc,
            opcode: console.apu().read_ram(p.pc as usize),
            operands: core::array::from_fn(|i| console.apu().read_ram(p.pc as usize + 1 + i)),
        }
    }
    fn jump_addr(&self, pc: usize) -> Option<usize> {
        None
    }
    fn addr(&self) -> String {
        format!("{:04X}", self.pc)
    }
    fn opcode_name(&self) -> String {
        ApuOpcodeData::from_opcode(self.opcode).name.to_string()
    }
    fn operands(&self, labels: &BTreeMap<usize, Label>) -> String {
        let d = ApuOpcodeData::from_opcode(self.opcode);
        format_address_modes(&d.addr_modes, &self.operands)
    }
}
