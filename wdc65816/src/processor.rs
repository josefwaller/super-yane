use crate::opcodes::*;
use crate::status_register::StatusRegister;

use std::default::Default;
use std::mem;

pub trait Memory {
    /// Read a single byte from memory
    fn read(&self, address: usize) -> u8;
    /// Write a single byte to memory
    fn write(&mut self, address: usize, value: u8);
}

#[derive(Default)]
pub struct Processor {
    /// Program Counter
    pc: u16,
    /// Program Bank Register
    pbr: u8,
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

/// Combine an 8-bit bank and a 16-bit address to form the final 24-bit address bus output
fn addr_bus_val(bank: u8, addr: u16) -> usize {
    bank as usize * 0x10000 + addr as usize
}

/// Read a single byte from a memory given an 8-bit bank and a 16-bit address
fn read_u8(memory: &mut impl Memory, bank: u8, addr: u16) -> u8 {
    // Combine bank and address to form final address
    memory.read(addr_bus_val(bank, addr))
}

fn read_u16<T: Memory>(memory: &mut T, bank: u8, addr: u16) -> u16 {
    // Read low first
    let low = read_u8(memory, bank, addr) as u16;
    let high = read_u8(memory, bank, addr.wrapping_add(1)) as u16;
    low + (high << 8)
}
fn read_u24<T: Memory>(memory: &mut T, bank: u8, addr: u16) -> u32 {
    // Read low first
    let lower = read_u16(memory, bank, addr) as u32;
    let high = read_u8(memory, bank, addr.wrapping_add(2)) as u32;
    lower + (high << 16)
}

const MSB_8: u32 = 0x80;
const MSB_16: u32 = 0x8000;
/// The value at which the bus overflows, i.e. 2^(24 + 1)
const BUS_OVERFLOW: usize = 0x1000000;

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }

    /// Add with Carry
    fn adc(&mut self, addr: usize, memory: &mut impl Memory) {
        // todo tidy
        // if self.p.is_8bit() {
        //     let value = memory.read(addr) as u16;
        //     let result: u32 = u32::from(self.p.c) + self.a as u32 + value as u32;
        //     self.p.n = (result & MSB_16 != 0).into();
        //     self.p.v = ((value ^ self.a) & (value ^ result as u16) & MSB_8 as u16 != 0).into();
        //     self.p.c = ((result & (MSB_16 << 1)) != 0).into();
        //     self.p.z = (result == 0).into();
        // } else {
        //     let value = read_u16(memory, addr) as u16;
        //     let result: u32 = u32::from(self.p.c) + self.a as u32 + value as u32;
        //     self.p.n = (result & MSB_8 != 0).into();
        //     self.p.v = ((value ^ self.a) & (value ^ result as u16) & MSB_8 as u16 != 0).into();
        //     self.p.c = ((result & (MSB_8 << 1)) != 0).into();
        //     self.p.z = (result == 0).into();
        // }
    }

    /// Individual methods for each addressing mode
    /// Combined with a CPU function to execute an instruction

    /// Immediate addressing
    fn i(&mut self, _memory: &mut impl Memory) -> usize {
        // Address is simply the next byte in the instruction
        let addr = addr_bus_val(self.pbr, self.pc);
        self.pc.wrapping_add(1);
        return addr;
    }
    /// Absolute addressing
    fn a(&mut self, memory: &mut impl Memory) -> usize {
        // Read the 16 bit address off the instruction
        let addr = read_u16(memory, self.pbr, self.pc);
        // Read the value at that address to get the final address
        let addr = read_u16(memory, self.dbr, addr);
        self.pc = self.pc.wrapping_add(2);
        return addr as usize;
    }
    /// Absolute X Indexed addressing
    fn ax(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self.a(memory);
        let addr = (addr + (self.x as usize)) % BUS_OVERFLOW;
        // Extra unused read for X indexed
        memory.read(self.pc.wrapping_sub(1) as usize);
        self.pc = self.pc.wrapping_add(2);
        return addr;
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl Memory) -> usize {
        let bank = read_u8(memory, self.pbr, self.pc);
        let addr = read_u16(memory, self.pbr, self.pc.wrapping_add(1));
        let addr = read_u24(memory, bank, addr) as usize;
        self.pc.wrapping_add(2);
        return addr;
    }
    /// Direct addressing
    fn d(&mut self, memory: &mut impl Memory) -> usize {
        let addr = read_u8(memory, self.pbr, self.pc) as usize;
        // Extra read if direct register low is not 0
        if self.d & 0xFF == 0 {
            read_u8(memory, self.pbr, self.pc);
        }
        let addr = (self.d as usize + addr) % BUS_OVERFLOW;
        self.pc.wrapping_add(1);
        return addr;
    }
    /// Direct Indirect addressing
    fn di(&mut self, memory: &mut impl Memory) -> usize {
        return 0;
    }

    /// Execute the next instruction in the program
    ///
    /// Read from the memory at the program counter to get the opcode,
    /// decode it, and execute it.
    /// Update the program counter accordingly.
    pub fn step<T: Memory>(&mut self, memory: &mut T) {
        macro_rules! cpu_func {
            ($func: ident, $get_addr: ident) => {{
                let addr = self.$get_addr(memory);
                self.$func(addr, memory)
            }};
        }
        let opcode = read_u8(memory, self.pbr, self.pc);
        self.pc += 1;

        match opcode {
            ADC_I => cpu_func!(adc, i),
            ADC_A => cpu_func!(adc, a),
            ADC_AL => cpu_func!(adc, al),
            ADC_AX => cpu_func!(adc, ax),
            ADC_D => cpu_func!(adc, d),
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
}
