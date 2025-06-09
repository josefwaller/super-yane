use crate::opcodes::*;
use crate::status_register::StatusRegister;

use std::default::Default;
use std::mem;

pub trait Memory {
    /// Read a single byte from memory
    fn read(&self, address: usize) -> u8;
    /// Write a single byte to memory
    fn write(&mut self, address: usize, value: u8);
    /// "Handle" an IO cycle
    fn io(&mut self);
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

const BUS_OVERFLOW: usize = 0x1000000;

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
fn read_u24(memory: &mut impl Memory, bank: u8, addr: u16) -> u32 {
    let low = read_u8(memory, bank, addr) as u32;
    let higher = read_u16(memory, bank, addr.wrapping_add(1)) as u32;
    low + (higher << 8)
}

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }

    /// Add with Carry
    fn adc(&mut self, addr: usize, memory: &mut impl Memory) {
        let (result, v, n) = if self.p.is_16bit() {
            let value = memory.read(addr) as u16 * 0x100 + memory.read(addr.wrapping_add(1)) as u16;
            let result = self.a.wrapping_add(value);
            let v = ((result ^ self.a) & (result ^ value)) & 0x8000 == 0;
            let n = result & 0x8000 != 0;
            (result, v, n)
        } else {
            let value = memory.read(addr);
            let result = (self.a as u8).wrapping_add(value);
            let v = ((result ^ self.a as u8) & (result ^ value)) & 0x80 == 0;
            let n = result & 0x80 != 0;
            (result as u16, v, n)
        };
        self.p.c = (result < self.a).into();
        self.p.v = v.into();
        self.p.n = n.into();
        self.p.z = (result == 0).into();
        self.a = result;
    }

    /// Individual methods for each addressing mode
    /// Combined with a CPU function to execute an instruction

    /// Immediate addressing
    fn i(&mut self, _memory: &mut impl Memory) -> usize {
        // Address is simply the next byte in the instruction
        let addr = addr_bus_val(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    /// Absolute addressing
    fn a(&mut self, memory: &mut impl Memory) -> usize {
        // Read the 16 bit address off the instruction
        let addr = read_u16(memory, self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr_bus_val(self.dbr, addr)
    }
    // Utility offset function for absolute indexed function
    fn a_off(&mut self, memory: &mut impl Memory, register: u16) -> usize {
        let addr = read_u16(memory, self.pbr, self.pc);
        let addr = addr.wrapping_add(register as u16);
        // Extra unused read for X indexed
        memory.io();
        self.pc = self.pc.wrapping_add(2);
        addr_bus_val(self.dbr, addr)
    }
    /// Absolute X Indexed addressing
    fn ax(&mut self, memory: &mut impl Memory) -> usize {
        self.a_off(memory, self.x)
    }
    /// Absolute Y Indexed addressing
    fn ay(&mut self, memory: &mut impl Memory) -> usize {
        self.a_off(memory, self.y)
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl Memory) -> usize {
        let bank = read_u8(memory, self.pbr, self.pc);
        let addr = read_u16(memory, self.pbr, self.pc.wrapping_add(1));
        self.pc = self.pc.wrapping_add(3);
        addr_bus_val(bank, addr)
    }
    /// Absolute Long X Indexed
    fn alx(&mut self, memory: &mut impl Memory) -> usize {
        let bank = read_u8(memory, self.pbr, self.pc);
        let addr = read_u16(memory, self.pbr, self.pc.wrapping_add(1)).wrapping_add(self.x);
        self.pc = self.pc.wrapping_add(3);
        addr_bus_val(bank, addr)
    }
    /// Direct addressing
    fn d(&mut self, memory: &mut impl Memory) -> usize {
        let addr = read_u8(memory, self.pbr, self.pc);
        // Extra read if direct register low is not 0
        if self.d & 0xFF != 0 {
            memory.io();
        }
        let addr = addr_bus_val(0x00, self.d + addr as u16);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    // Direct addressing with offset
    fn d_off(&mut self, memory: &mut impl Memory, register: u16) -> usize {
        let addr = self.d(memory) as u16;
        memory.io();
        addr_bus_val(0x00, addr.wrapping_add(register))
    }
    /// Direct X Indexed addressing
    fn dx(&mut self, memory: &mut impl Memory) -> usize {
        self.d_off(memory, self.x)
    }
    /// Direct Y Indexed addressing
    fn dy(&mut self, memory: &mut impl Memory) -> usize {
        self.d_off(memory, self.y)
    }
    /// Direct Indirect addressing
    fn di(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self.d(memory);
        addr_bus_val(self.dbr, addr as u16)
    }
    /// Direct Indirect X Indexed addressing
    fn dix(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self.d(memory) as u16;
        memory.io();
        addr_bus_val(self.dbr, addr.wrapping_add(self.x as u16))
    }
    /// Direct Indirect Y Indexed addressing
    fn diy(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self.di(memory);
        memory.io();
        (addr + self.y as usize) % BUS_OVERFLOW
    }
    /// Direct Indirect Long addressing
    fn dil(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self
            .d
            .wrapping_add(read_u8(memory, self.pbr, self.pc) as u16);
        // Read the value of the pointer from memory
        read_u24(memory, 0x00, addr) as usize
    }
    /// Direct Indirect Long Y Indexed addressing
    fn dily(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self.dil(memory);
        (addr + self.y as usize) % BUS_OVERFLOW
    }
    /// Stack Relative addressing
    fn sr(&mut self, memory: &mut impl Memory) -> usize {
        let addr = self
            .s
            .wrapping_add(read_u8(memory, self.pbr, self.pc) as u16);
        memory.io();
        addr_bus_val(0x0, addr)
    }
    /// Stack Reslative Indirect Y Indexed addressing
    fn sriy(&mut self, memory: &mut impl Memory) -> usize {
        let addr = addr_bus_val(self.dbr, self.sr(memory) as u16);
        (addr + self.y as usize) % BUS_OVERFLOW
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
            ADC_AX => cpu_func!(adc, ax),
            ADC_AY => cpu_func!(adc, ay),
            ADC_AL => cpu_func!(adc, al),
            ADC_ALX => cpu_func!(adc, alx),
            ADC_D => cpu_func!(adc, d),
            ADC_DX => cpu_func!(adc, dx),
            ADC_DI => cpu_func!(adc, di),
            ADC_DIX => cpu_func!(adc, dix),
            ADC_DIY => cpu_func!(adc, diy),
            ADC_DIL => cpu_func!(adc, dil),
            ADC_DILY => cpu_func!(adc, dily),
            ADC_SR => cpu_func!(adc, sr),
            ADC_SRIY => cpu_func!(adc, sriy),
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
}
