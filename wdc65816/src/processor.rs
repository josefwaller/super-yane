use crate::opcodes::*;
use crate::status_register::StatusRegister;

use std::default::Default;

pub trait Memory {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
}

#[derive(Default)]
pub struct Processor {
    /// Program Counnter
    pc: usize,
    /// Accumulator
    a: u16,
    /// X Register
    x: u16,
    /// Y Register
    y: u16,
    /// Status Register
    p: StatusRegister,
    /// Direct Register
    d: u16,
    /// Data Bank Register
    dbr: u8,
    /// Stack Pointer
    s: u16,
}

const MSB_8: u32 = 0x80;
const MSB_16: u32 = 0x8000;

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }
    /// Add with Carry
    fn adc(&mut self, value: u16) {
        let result: u32 = u32::from(self.p.c) + self.a as u32 + value as u32;
        self.p.z = (result == 0).into();
        if self.p.m == 0 {
            self.p.n = (result & MSB_16 != 0).into();
            self.p.v = ((value ^ self.a) & (value ^ result as u16) & MSB_8 as u16 != 0).into();
            self.p.c = ((result & (MSB_16 << 1)) != 0).into();
        } else {
            self.p.n = (result & MSB_8 != 0).into();
            self.p.v = ((value ^ self.a) & (value ^ result as u16) & MSB_8 as u16 != 0).into();
            self.p.c = ((result & (MSB_8 << 1)) != 0).into();
        }
    }

    /// Execute the next instruction in the program
    ///
    /// Read from the memory at the program counter to get the opcode,
    /// decode it, and execute it.
    /// Update the program counter accordingly.
    pub fn step<T: Memory>(&mut self, memory: &mut T) {
        /// Immediate function
        macro_rules! i {
            ($f: ident) => {{
                if self.p.m.into() {
                    self.$f(memory.read(self.pc) as u16);
                    self.pc += 1;
                } else {
                    // Read lower byte first
                    let low = memory.read(self.pc) as u16;
                    let addr = low + memory.read(self.pc + 1) as u16 * 0x100;
                    self.$f(addr)
                }
            }};
        }
        let opcode = memory.read(self.pc);
        self.pc += 1;

        match opcode {
            ADC_I => i!(adc),
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
}
