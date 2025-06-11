use crate::bus::Bus;
use crate::flag::Flag;
use crate::opcodes::*;
use crate::status_register::StatusRegister;
use crate::u24::u24;

use std::default::Default;

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
    /// Lower byte of the accumulator
    a: u8,
    /// Upper byte of the accumulator
    b: u8,
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

/// Read a single byte from a memory given an 8-bit bank and a 16-bit address
fn read_u8(memory: &mut impl Memory, addr: Bus) -> u8 {
    // Combine bank and address to form final address
    memory.read(addr.into())
}

fn read_u16<T: Memory>(memory: &mut T, addr: Bus) -> u16 {
    // Read low first
    let low = read_u8(memory, addr) as u16;
    let high = read_u8(memory, addr.offset(1)) as u16;
    low + (high << 8)
}
fn read_u24(memory: &mut impl Memory, addr: Bus) -> u24 {
    let low = read_u8(memory, addr) as u32;
    let higher = read_u16(memory, addr.offset(1)) as u32;
    u24::from(low + (higher << 8))
}

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }

    /// Add two bytes together and calculate flags
    /// Returns in the format (value, carry, zero, negative, (signed) overflow)
    fn add_bytes(a: u8, b: u8, carry: Flag) -> (u8, Flag, Flag, Flag, Flag) {
        let result = a.wrapping_add(b).wrapping_add(carry.into());
        return (
            result,
            // Carry flag
            (result < a).into(),
            // Zero flag
            (result == 0).into(),
            // Negative flag
            ((result & 0x80) == 0x80).into(),
            // Overflow flag
            (((result ^ a as u8) & (result ^ b)) & 0x80 == 0).into(),
        );
    }

    /// Add with Carry
    fn adc(&mut self, addr: Bus, memory: &mut impl Memory) {
        let addr: u24 = addr.into();
        let (value, c, z, n, v) = Processor::add_bytes(self.a, memory.read(addr.into()), self.p.c);
        self.a = value;
        if self.p.is_16bit() {
            let (value, c, z2, n, v) =
                Processor::add_bytes(self.b, memory.read(addr.wrapping_add(1u32).into()), c);
            self.b = value;
            // Both need to be 0 for the zero flag to be set
            self.p.z = (z & z2).into();
            self.p.n = n.into();
            self.p.v = v.into();
            self.p.c = c.into();
        } else {
            self.p.z = z.into();
            self.p.n = n.into();
            self.p.v = v.into();
            self.p.c = c.into();
        }
    }

    /// Individual methods for each addressing mode
    /// Combined with a CPU function to execute an instruction
    /// All return (bank, address) which are combined to form the final address in the
    /// cpu function to form the final 24-bit address

    /// Immediate addressing
    fn i(&mut self, _memory: &mut impl Memory) -> Bus {
        // Address is simply the next byte in the instruction
        let addr = Bus::new(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    /// Absolute addressing
    fn a(&mut self, memory: &mut impl Memory) -> Bus {
        // Read the 16 bit address off the instruction
        let addr = read_u16(memory, Bus::new(self.pbr, self.pc));
        self.pc = self.pc.wrapping_add(2);
        Bus::new(self.dbr, addr)
    }
    // Utility offset function for absolute indexed function
    fn a_off(&mut self, memory: &mut impl Memory, register: u16) -> Bus {
        let addr = read_u16(memory, Bus::new(self.pbr, self.pc));
        let addr = addr.wrapping_add(register as u16);
        // Extra unused read for X indexed
        memory.io();
        self.pc = self.pc.wrapping_add(2);
        Bus::new(self.dbr, addr)
    }
    /// Absolute X Indexed addressing
    fn ax(&mut self, memory: &mut impl Memory) -> Bus {
        self.a_off(memory, self.x)
    }
    /// Absolute Y Indexed addressing
    fn ay(&mut self, memory: &mut impl Memory) -> Bus {
        self.a_off(memory, self.y)
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = Bus::new(self.pbr, self.pc);
        let bank = read_u8(memory, addr);
        let addr = read_u16(memory, addr.offset(1));
        self.pc = self.pc.wrapping_add(3);
        Bus::new(bank, addr)
    }
    /// Absolute Long X Indexed
    fn alx(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = Bus::new(self.pbr, self.pc);
        let bank = read_u8(memory, addr);
        let addr = read_u16(memory, addr.offset(1)).wrapping_add(self.x);
        self.pc = self.pc.wrapping_add(3);
        Bus::new(bank, addr)
    }
    /// Direct addressing
    fn d(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = read_u8(memory, Bus::new(self.pbr, self.pc));
        // Extra read if direct register low is not 0
        if self.d & 0xFF != 0 {
            memory.io();
        }
        self.pc = self.pc.wrapping_add(1);
        Bus::new(0x00, self.d + addr as u16)
    }
    // Direct addressing with offset
    fn d_off(&mut self, memory: &mut impl Memory, register: u16) -> Bus {
        let addr = self.d(memory);
        memory.io();
        addr.offset(register)
    }
    /// Direct X Indexed addressing
    fn dx(&mut self, memory: &mut impl Memory) -> Bus {
        self.d_off(memory, self.x)
    }
    /// Direct Y Indexed addressing
    fn dy(&mut self, memory: &mut impl Memory) -> Bus {
        self.d_off(memory, self.y)
    }
    /// Direct Indirect addressing
    fn di(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = self.d(memory);
        Bus::new(self.dbr, addr.address)
    }
    /// Direct Indirect X Indexed addressing
    fn dix(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = self.di(memory);
        memory.io();
        addr.offset(self.x)
    }
    /// Direct Indirect Y Indexed addressing
    fn diy(&mut self, memory: &mut impl Memory) -> Bus {
        // Have to manually build and deconstruct this one
        let addr: u24 = self.di(memory).into();
        memory.io();
        let addr = addr.wrapping_add(self.y);
        addr.into()
    }
    /// Direct Indirect Long addressing
    fn dil(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = self
            .d
            .wrapping_add(read_u8(memory, Bus::new(self.pbr, self.pc)) as u16);
        // Read the value of the pointer from memory
        let addr = read_u24(memory, Bus::new(0x00, addr));
        addr.into()
    }
    /// Direct Indirect Long Y Indexed addressing
    fn dily(&mut self, memory: &mut impl Memory) -> Bus {
        let addr: u24 = self.dil(memory).into();
        addr.wrapping_add(self.y).into()
    }
    /// Stack Relative addressing
    fn sr(&mut self, memory: &mut impl Memory) -> Bus {
        let addr = self
            .s
            .wrapping_add(read_u8(memory, Bus::new(self.pbr, self.pc)) as u16);
        memory.io();
        self.pc = self.pc.wrapping_add(1);
        Bus::new(0x0, addr)
    }
    /// Stack Reslative Indirect Y Indexed addressing
    fn sriy(&mut self, memory: &mut impl Memory) -> Bus {
        let addr: u24 = self.sr(memory).with_bank(self.dbr).into();
        addr.wrapping_add(self.y).into()
    }

    /// Execute the next instruction in the program
    ///
    /// Read from the memory at the program counter to get the opcode,
    /// decode it, and execute it.
    /// Update the program counter accordingly.
    pub fn step<T: Memory>(&mut self, memory: &mut T) {
        macro_rules! cpu_func {
            ($func: ident, $get_addr: ident) => {{
                let bus = self.$get_addr(memory);
                self.$func(bus, memory)
            }};
        }
        let opcode = read_u8(memory, Bus::new(self.pbr, self.pc));
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
