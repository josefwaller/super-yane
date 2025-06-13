use crate::opcodes::*;
use crate::status_register::StatusRegister;
use crate::u24::u24;

use std::default::Default;
use std::sync::Arc;

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
    /// X Register low
    xl: u8,
    /// X Register high
    xh: u8,
    /// Y Register low
    yl: u8,
    /// Y Register high
    yh: u8,
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
fn read_u8(memory: &mut impl Memory, addr: u24) -> u8 {
    // Combine bank and address to form final address
    memory.read(addr.into())
}

fn read_u16<T: Memory>(memory: &mut T, addr: u24) -> u16 {
    // Read low first
    let low = read_u8(memory, addr) as u16;
    let high = read_u8(memory, addr.wrapping_add(1u32)) as u16;
    low + (high << 8)
}
fn read_u24(memory: &mut impl Memory, addr: u24) -> u24 {
    let low = read_u16(memory, addr);
    let high = read_u8(memory, addr.wrapping_add(2u32));
    u24::from(high, low)
}

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }
    /// Get the X register as a u16
    /// If the X register is 8-bit, only the bottom 8-bits will be used
    fn x(&self) -> u16 {
        if self.p.xy_is_16bit() {
            self.xh as u16 * 0x100 + self.xl as u16
        } else {
            self.xl as u16
        }
    }
    /// Get the Y register as a u16
    /// If the Y register is 8-bit, only the bottom 8-bits will be used
    fn y(&self) -> u16 {
        if self.p.xy_is_16bit() {
            self.yh as u16 * 0x100 + self.yl as u16
        } else {
            self.yl as u16
        }
    }
    /// Push a single byte to stack
    fn push_u8(&mut self, value: u8, memory: &mut impl Memory) {
        memory.write(self.s.into(), value);
        self.s = self.s.wrapping_add(1);
        // Force high to 0 in emulation mode
        if self.p.e {
            self.s = self.s & 0xFF;
        }
    }
    /// Push a 16-bit value to the stack
    fn push_u16(&mut self, value: u16, memory: &mut impl Memory) {
        // Push high first
        memory.write(self.s.into(), (value >> 8) as u8);
        memory.write(self.s.into(), (value & 0xFF) as u8);
        self.s = self.s.wrapping_add(2);
        // Force high to 0 in emulation mode
        if self.p.e {
            self.s = self.s & 0xFF
        }
    }
    /// Add with Carry 8-bit
    fn adc_8(&mut self, value: u8) {
        let (a, c) = self.a.overflowing_add(value);
        self.p.v = ((self.a ^ a as u8) & (value ^ a)) & 0x80 == 0;
        self.p.n = (a & 0x80) != 0;
        self.p.z = a == 0;
        self.p.c = c;
        self.a = a;
    }
    /// Add with Carry 16-bit
    fn adc_16(&mut self, low: u8, high: u8) {
        let (a, c) = self.a.overflowing_add(low);
        self.a = a;
        let (b, c2) = self.b.overflowing_add(high);
        self.p.n = (b & 0x80) != 0;
        self.p.z = a == 0 && b == 0;
        self.p.c = c2;
        self.a = a;
        self.p.v = ((self.b ^ b as u8) & (high ^ b)) & 0x80 == 0;
        self.b = b.wrapping_add(c.into());
    }
    /// And 8-bit
    fn and_8(&mut self, value: u8) {
        self.a = self.a & value;
        self.p.n = self.a > 0x7F;
        self.p.z = self.a == 0;
    }
    /// And 16-bit
    fn and_16(&mut self, low: u8, high: u8) {
        self.a = self.a & low;
        self.b = self.b & high;
        self.p.n = self.b > 0x7F;
        self.p.z = self.a == 0 && self.b == 0;
    }
    /// ASL 8-bit
    fn asl_8(&mut self, value: u8) -> u8 {
        let (value, carry) = value.overflowing_shl(1);
        self.p.c = carry;
        self.p.n = value > 0x7F;
        self.p.z = value == 0;
        value
    }
    /// ASL 16-bit
    fn asl_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let low = self.asl_8(low);
        let carry = self.p.c;
        let zero = self.p.z;
        let high = self.asl_8(high).wrapping_add(carry.into());
        // Both need to be 0
        self.p.z = self.p.z & zero;
        (low, high)
    }

    /// Branch
    fn branch(&mut self, memory: &mut impl Memory, offset: i16) {
        memory.io();
        self.pc = self.pc.wrapping_add_signed(offset.into());
    }

    /// Bit 8-bit
    fn bit_8(&mut self, value: u8) {
        self.p.n = value > 0x7F;
        self.p.v = (value & 0x40) != 0;
        self.p.z = (value & self.a) == 0;
    }
    /// Bit 16-bit
    fn bit_16(&mut self, low: u8, high: u8) {
        self.p.n = high > 0x7F;
        self.p.v = (high & 0x40) != 0;
        self.p.z = ((high & self.b) | (low & self.a)) == 0;
    }
    /// Bit Immediate 8-bit
    /// See the etry for BIT_I
    fn bit_i_8(&mut self, value: u8) {
        self.p.z = (value & self.a) == 0;
    }
    /// Bit Immediate 16-bit
    fn bit_i_16(&mut self, low: u8, high: u8) {
        self.p.z = ((self.a & low) | (self.b & high)) == 0;
    }
    /// Generic compare function used for CMP, CPX, CPY 8-bit
    fn compare_8(&mut self, a: u8, b: u8) {
        let (result, carry) = a.overflowing_sub(b);
        self.p.n = result > 0x7F;
        self.p.z = result == 0;
        self.p.c = !carry;
    }
    /// Generic compare function used for CMP, CPX, CPY 16-bit
    fn compare_16(&mut self, (a_low, a_high): (u8, u8), (b_low, b_high): (u8, u8)) {
        let (result, carry) = (a_high as u16 * 0x100 + a_low as u16)
            .overflowing_sub(b_high as u16 * 0x100 + b_low as u16);
        self.p.n = result > 0x7FFF;
        self.p.z = result == 0;
        self.p.c = !carry;
    }

    /// Compare (CMP) 8-bit
    fn cmp_8(&mut self, value: u8) {
        self.compare_8(self.a, value);
    }
    fn cmp_16(&mut self, low: u8, high: u8) {
        self.compare_16((self.a, self.b), (low, high));
    }
    /// Compare X (CPX) 8-bit
    fn cpx_8(&mut self, value: u8) {
        self.compare_8(self.xl, value);
    }
    /// Compare X (CPX) 16-bit
    fn cpx_16(&mut self, low: u8, high: u8) {
        self.compare_16((self.xl, self.xh), (low, high));
    }
    /// Compare Y (CPY) 8-bit
    fn cpy_8(&mut self, value: u8) {
        self.compare_8(self.yl, value);
    }
    /// Compare Y (CPY) 16-bit
    fn cpy_16(&mut self, low: u8, high: u8) {
        self.compare_16((self.yl, self.yh), (low, high));
    }

    /// Break to a given address
    fn break_to(&mut self, memory: &mut impl Memory, addr_16: u16, addr_8: u16, set_b: bool) {
        if self.p.e {
            // Since we already incremented the PC by 1 we want to just add 1
            self.push_u16(self.pc.wrapping_add(1), memory);
            // Clone processor register to set B flag
            let mut p = self.p;
            if set_b {
                p.b = true;
            }
            self.push_u8(p.to_byte(), memory);
            self.pbr = 0x00;
            self.pc = addr_8;
        } else {
            self.push_u8(self.pbr, memory);
            self.push_u16(self.pc.wrapping_add(1), memory);
            self.push_u8(self.p.to_byte(), memory);
            self.pbr = 0x00;
            self.pc = addr_16;
        }
    }
    /// Decrement (DEC) 8-bit
    fn dec_8(&mut self, value: u8) -> u8 {
        let r = value.wrapping_sub(1);
        self.p.n = (r & 0x80) == 0;
        self.p.z = r == 0;
        r
    }
    /// Decrement (DEC) 16-bit
    fn dec_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let (low, carry) = low.overflowing_sub(1);
        let high = high.wrapping_sub(1).wrapping_sub(carry.into());
        self.p.n = (high & 0x80) == 0;
        self.p.z = (high == 0) && (low == 0);
        (low, high)
    }

    /// Individual methods for each addressing mode
    /// Combined with a CPU function to execute an instruction
    /// All return (bank, address) which are combined to form the final address in the
    /// cpu function to form the final 24-bit address

    /// Immediate addressing
    fn i(&mut self, _memory: &mut impl Memory) -> u24 {
        // Address is simply the next byte in the instruction
        let addr = u24::from(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    /// Absolute addressing
    fn a(&mut self, memory: &mut impl Memory) -> u24 {
        // Read the 16 bit address off the instruction
        let addr = read_u16(memory, u24::from(self.pbr, self.pc));
        self.pc = self.pc.wrapping_add(2);
        u24::from(self.dbr, addr)
    }
    // Utility offset function for absolute indexed function
    fn a_off(&mut self, memory: &mut impl Memory, register: u16) -> u24 {
        let addr = read_u16(memory, u24::from(self.pbr, self.pc));
        let addr = addr.wrapping_add(register as u16);
        // Extra unused read for X indexed
        memory.io();
        self.pc = self.pc.wrapping_add(2);
        u24::from(self.dbr, addr)
    }
    /// Absolute X Indexed addressing
    fn ax(&mut self, memory: &mut impl Memory) -> u24 {
        self.a_off(memory, self.x())
    }
    /// Absolute Y Indexed addressing
    fn ay(&mut self, memory: &mut impl Memory) -> u24 {
        self.a_off(memory, self.y())
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = u24::from(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(3);
        read_u24(memory, addr)
    }
    /// Absolute Long X Indexed
    fn alx(&mut self, memory: &mut impl Memory) -> u24 {
        self.al(memory).wrapping_add(self.x())
    }
    /// Direct addressing
    fn d(&mut self, memory: &mut impl Memory) -> u24 {
        let offset = read_u8(memory, u24::from(self.pbr, self.pc));
        // Extra cycle if direct register low is not 0
        if self.d & 0xFF != 0 {
            memory.io();
        }
        self.pc = self.pc.wrapping_add(1);
        u24::from(0x00, self.d.wrapping_add(offset as u16))
    }
    // Direct addressing with offset
    fn d_off(&mut self, memory: &mut impl Memory, register: u16) -> u24 {
        let addr = self.d(memory);
        memory.io();
        addr.wrapping_add(register).with_bank(0x00)
    }
    /// Direct X Indexed addressing
    fn dx(&mut self, memory: &mut impl Memory) -> u24 {
        self.d_off(memory, self.x())
    }
    /// Direct Y Indexed addressing
    fn dy(&mut self, memory: &mut impl Memory) -> u24 {
        self.d_off(memory, self.y())
    }
    /// Direct Indirect addressing
    fn di(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = self.d(memory);
        addr.with_bank(self.dbr)
    }
    /// Direct Indirect X Indexed addressing
    fn dix(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = self.di(memory);
        memory.io();
        addr.wrapping_add(self.x()).with_bank(self.dbr)
    }
    /// Direct Indirect Y Indexed addressing
    fn diy(&mut self, memory: &mut impl Memory) -> u24 {
        let addr: u24 = self.di(memory).into();
        memory.io();
        addr.wrapping_add(self.y())
    }
    /// Direct Indirect Long addressing
    fn dil(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = self
            .d
            .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16);
        // Read the value of the pointer from memory
        read_u24(memory, u24::from(0x00, addr))
    }
    /// Direct Indirect Long Y Indexed addressing
    fn dily(&mut self, memory: &mut impl Memory) -> u24 {
        let addr: u24 = self.dil(memory).into();
        addr.wrapping_add(self.y())
    }
    /// Stack Relative addressing
    fn sr(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = self
            .s
            .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16);
        memory.io();
        self.pc = self.pc.wrapping_add(1);
        u24::from(0x0, addr)
    }
    /// Stack Reslative Indirect Y Indexed addressing
    fn sriy(&mut self, memory: &mut impl Memory) -> u24 {
        let addr: u24 = self.sr(memory).with_bank(self.dbr).into();
        addr.wrapping_add(self.y())
    }

    /// Execute the next instruction in the program
    ///
    /// Read from the memory at the program counter to get the opcode,
    /// decode it, and execute it.
    /// Update the program counter accordingly.
    pub fn step<T: Memory>(&mut self, memory: &mut T) {
        macro_rules! read_func {
            ($f_8: ident, $f_16: ident, $addr: ident) => {{
                let addr = self.$addr(memory);
                if self.p.a_is_8bit() {
                    self.$f_8(memory.read(addr.into()));
                } else {
                    self.$f_16(
                        memory.read(addr.into()),
                        memory.read(addr.wrapping_add(1u32).into()),
                    );
                }
            }};
        }
        macro_rules! read_write_func {
            ($func_8: ident, $func_16: ident, $get_addr: ident) => {{
                let address = self.$get_addr(memory);
                if self.p.a_is_8bit() {
                    let value = self.$func_8(memory.read(address.into()));
                    memory.write(address.into(), value);
                } else {
                    let (low, high) = self.$func_16(
                        memory.read(address.into()),
                        memory.read(address.wrapping_add(1u32).into()),
                    );
                    memory.write(address.into(), low);
                    memory.write(address.wrapping_add(1u32).into(), high);
                }
            }};
        }
        macro_rules! branch_if {
            ($flag: ident, $value: expr) => {{
                let offset = read_u8(memory, u24::from(self.pbr, self.pc.wrapping_add(1))) as i16;
                if self.p.$flag == $value {
                    self.branch(memory, offset);
                }
            }};
        }
        macro_rules! set_flag {
            ($flag: ident, $value: expr) => {{
                memory.io();
                self.p.$flag = $value;
            }};
        }
        macro_rules! reg_func {
            ($rl: ident, $rh: ident, $is_16: ident, $f_8: ident, $f_16: ident) => {{
                if self.p.$is_16() {
                    let (low, high) = self.$f_16(self.$rl, self.$rh);
                    self.$rl = low;
                    self.$rh = high;
                } else {
                    self.$rl = self.$f_8(self.$rl);
                }
            }};
        }
        let opcode = read_u8(memory, u24::from(self.pbr, self.pc));
        self.pc += 1;

        match opcode {
            ADC_I => read_func!(adc_8, adc_16, i),
            ADC_A => read_func!(adc_8, adc_16, a),
            ADC_AX => read_func!(adc_8, adc_16, ax),
            ADC_AY => read_func!(adc_8, adc_16, ay),
            ADC_AL => read_func!(adc_8, adc_16, al),
            ADC_ALX => read_func!(adc_8, adc_16, alx),
            ADC_D => read_func!(adc_8, adc_16, d),
            ADC_DX => read_func!(adc_8, adc_16, dx),
            ADC_DI => read_func!(adc_8, adc_16, di),
            ADC_DIX => read_func!(adc_8, adc_16, dix),
            ADC_DIY => read_func!(adc_8, adc_16, diy),
            ADC_DIL => read_func!(adc_8, adc_16, dil),
            ADC_DILY => read_func!(adc_8, adc_16, dily),
            ADC_SR => read_func!(adc_8, adc_16, sr),
            ADC_SRIY => read_func!(adc_8, adc_16, sriy),
            AND_I => read_func!(and_8, and_16, i),
            AND_A => read_func!(and_8, and_16, a),
            AND_AL => read_func!(and_8, and_16, al),
            AND_D => read_func!(and_8, and_16, d),
            AND_DI => read_func!(and_8, and_16, di),
            AND_DIL => read_func!(and_8, and_16, dil),
            AND_AX => read_func!(and_8, and_16, ax),
            AND_ALX => read_func!(and_8, and_16, alx),
            AND_AY => read_func!(and_8, and_16, ay),
            AND_DX => read_func!(and_8, and_16, dx),
            AND_DIX => read_func!(and_8, and_16, dix),
            AND_DIY => read_func!(and_8, and_16, diy),
            AND_DILY => read_func!(and_8, and_16, dily),
            AND_SR => read_func!(and_8, and_16, sr),
            AND_SRIY => read_func!(and_8, and_16, sriy),
            ASL_ACC => {
                self.a = self.asl_8(self.a);
                if self.p.a_is_16bit() {
                    let carry = self.p.c;
                    self.b = self.asl_8(self.b).wrapping_add(carry.into());
                }
            }
            ASL_A => read_write_func!(asl_8, asl_16, a),
            ASL_D => read_write_func!(asl_8, asl_16, d),
            ASL_AX => read_write_func!(asl_8, asl_16, ax),
            ASL_DX => read_write_func!(asl_8, asl_16, dx),
            BCC => branch_if!(c, false),
            BCS => branch_if!(c, true),
            BEQ => branch_if!(z, true),
            BIT_I => {
                // > Immediate addressing only affects the z flag (with the result of the bitwise And), but does not affect the n and v flags.
                // > All other addressing modes of BIT affect the n, v, and z flags.
                // > This is the only instruction in the 6502 family where the flags affected depends on the addressing mode.
                // http://www.6502.org/tutorials/65c816opcodes.html#6.1.2.2
                read_func!(bit_i_8, bit_i_16, i);
            }
            BIT_A => read_func!(bit_8, bit_16, a),
            BIT_D => read_func!(bit_8, bit_16, d),
            BIT_AX => read_func!(bit_8, bit_16, ax),
            BIT_DX => read_func!(bit_8, bit_16, dx),
            BMI => branch_if!(n, true),
            BNE => branch_if!(z, false),
            BPL => branch_if!(n, false),
            BRA => {
                let addr = read_u8(memory, u24::from(self.pbr, self.pc)) as i16;
                self.branch(memory, addr);
            }
            // BRK => self.brk(),
            BRL => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc)) as i16;
                self.branch(memory, addr);
            }
            BVC => branch_if!(v, false),
            BVS => branch_if!(v, true),
            CLC => set_flag!(c, false),
            CLD => set_flag!(d, false),
            CLI => set_flag!(i, false),
            CLV => set_flag!(v, false),
            CMP_I => read_func!(cmp_8, cmp_16, i),
            CMP_A => read_func!(cmp_8, cmp_16, a),
            CMP_AL => read_func!(cmp_8, cmp_16, al),
            CMP_D => read_func!(cmp_8, cmp_16, d),
            CMP_DI => read_func!(cmp_8, cmp_16, di),
            CMP_DIL => read_func!(cmp_8, cmp_16, dil),
            CMP_AX => read_func!(cmp_8, cmp_16, ax),
            CMP_ALX => read_func!(cmp_8, cmp_16, alx),
            CMP_AY => read_func!(cmp_8, cmp_16, ay),
            CMP_DX => read_func!(cmp_8, cmp_16, dx),
            CMP_DIX => read_func!(cmp_8, cmp_16, dix),
            CMP_DIY => read_func!(cmp_8, cmp_16, diy),
            CMP_DILY => read_func!(cmp_8, cmp_16, dily),
            CMP_SR => read_func!(cmp_8, cmp_16, sr),
            CMP_SRIY => read_func!(cmp_8, cmp_16, sriy),
            COP => self.break_to(memory, 0xFFE4, 0xFFF4, false),
            CPX_I => read_func!(cpx_8, cpx_16, i),
            CPX_A => read_func!(cpx_8, cpx_16, a),
            CPX_D => read_func!(cpx_8, cpx_16, d),
            CPY_I => read_func!(cpy_8, cpy_16, i),
            CPY_A => read_func!(cpy_8, cpy_16, a),
            CPY_D => read_func!(cpy_8, cpy_16, d),
            DEC_ACC => reg_func!(a, b, a_is_16bit, dec_8, dec_16),
            DEC_A => read_write_func!(dec_8, dec_16, a),
            DEC_D => read_write_func!(dec_8, dec_16, d),
            DEC_AX => read_write_func!(dec_8, dec_16, ax),
            DEC_DX => read_write_func!(dec_8, dec_16, dx),
            DEX => reg_func!(xl, xh, xy_is_16bit, dec_8, dec_16),
            DEY => reg_func!(yl, yh, xy_is_16bit, dec_8, dec_16),
            /*EOR_I => cpu_func!(eor_8, eor_16, i),
            EOR_A => cpu_func!(eor_8, eor_16, a),
            EOR_AL => cpu_func!(eor_8, eor_16, al),
            EOR_D => cpu_func!(eor_8, eor_16, d),
            EOR_DI => cpu_func!(eor_8, eor_16, di),
            EOR_DIL => cpu_func!(eor_8, eor_16, dil),
            EOR_AX => cpu_func!(eor_8, eor_16, ax),
            EOR_ALX => cpu_func!(eor_8, eor_16, alx),
            EOR_AY => cpu_func!(eor_8, eor_16, ay),
            EOR_DX => cpu_func!(eor_8, eor_16, dx),
            EOR_DIX => cpu_func!(eor_8, eor_16, dix),
            EOR_DIY => cpu_func!(eor_8, eor_16, diy),
            EOR_DILY => cpu_func!(eor_8, eor_16, dily),
            EOR_SR => cpu_func!(eor_8, eor_16, sr),
            EOR_SRIY => cpu_func!(eor_8, eor_16, sriy),
            INC_ACC => cpu_func!(inc_8, inc_16, acc),
            INC_A => cpu_func!(inc_8, inc_16, a),
            INC_D => cpu_func!(inc_8, inc_16, d),
            INC_AX => cpu_func!(inc_8, inc_16, ax),
            INC_DX => cpu_func!(inc_8, inc_16, dx),
            INX => self.inx(),
            INY => self.iny(),
            JMP_A => cpu_func!(jmp_8, jmp_16, a),
            JMP_AI => cpu_func!(jmp_8, jmp_16, ai),
            JMP_AIX => cpu_func!(jmp_8, jmp_16, aix),
            JMP_AL => cpu_func!(jmp_8, jmp_16, al),
            JMP_AIL => cpu_func!(jmp_8, jmp_16, ail),
            JSR_A => cpu_func!(jsr_8, jsr_16, a),
            JSR_AIX => cpu_func!(jsr_8, jsr_16, aix),
            JSR_AL => cpu_func!(jsr_8, jsr_16, al),
            LDA_I => cpu_func!(lda_8, lda_16, i),
            LDA_A => cpu_func!(lda_8, lda_16, a),
            LDA_AL => cpu_func!(lda_8, lda_16, al),
            LDA_D => cpu_func!(lda_8, lda_16, d),
            LDA_DI => cpu_func!(lda_8, lda_16, di),
            LDA_DIL => cpu_func!(lda_8, lda_16, dil),
            LDA_AX => cpu_func!(lda_8, lda_16, ax),
            LDA_ALX => cpu_func!(lda_8, lda_16, alx),
            LDA_AY => cpu_func!(lda_8, lda_16, ay),
            LDA_DX => cpu_func!(lda_8, lda_16, dx),
            LDA_DIX => cpu_func!(lda_8, lda_16, dix),
            LDA_DIY => cpu_func!(lda_8, lda_16, diy),
            LDA_DILY => cpu_func!(lda_8, lda_16, dily),
            LDA_SR => cpu_func!(lda_8, lda_16, sr),
            LDA_SRIY => cpu_func!(lda_8, lda_16, sriy),
            LDX_I => cpu_func!(ldx_8, ldx_16, i),
            LDX_A => cpu_func!(ldx_8, ldx_16, a),
            LDX_D => cpu_func!(ldx_8, ldx_16, d),
            LDX_AY => cpu_func!(ldx_8, ldx_16, ay),
            LDX_DY => cpu_func!(ldx_8, ldx_16, dy),
            LDY_I => cpu_func!(ldy_8, ldy_16, i),
            LDY_A => cpu_func!(ldy_8, ldy_16, a),
            LDY_D => cpu_func!(ldy_8, ldy_16, d),
            LDY_AX => cpu_func!(ldy_8, ldy_16, ax),
            LDY_DX => cpu_func!(ldy_8, ldy_16, dx),
            LSR_ACC => cpu_func!(lsr_8, lsr_16, acc),
            LSR_A => cpu_func!(lsr_8, lsr_16, a),
            LSR_D => cpu_func!(lsr_8, lsr_16, d),
            LSR_AX => cpu_func!(lsr_8, lsr_16, ax),
            LSR_DX => cpu_func!(lsr_8, lsr_16, dx),
            MVN_NEXT => cpu_func!(mvn_8, mvn_16, next),
            MVN_PREV => cpu_func!(mvn_8, mvn_16, prev),
            NOP => self.nop(),
            ORA_I => cpu_func!(ora_8, ora_16, i),
            ORA_A => cpu_func!(ora_8, ora_16, a),
            ORA_AL => cpu_func!(ora_8, ora_16, al),
            ORA_D => cpu_func!(ora_8, ora_16, d),
            ORA_DI => cpu_func!(ora_8, ora_16, di),
            ORA_DIL => cpu_func!(ora_8, ora_16, dil),
            ORA_AX => cpu_func!(ora_8, ora_16, ax),
            ORA_ALX => cpu_func!(ora_8, ora_16, alx),
            ORA_AY => cpu_func!(ora_8, ora_16, ay),
            ORA_DX => cpu_func!(ora_8, ora_16, dx),
            ORA_DIX => cpu_func!(ora_8, ora_16, dix),
            ORA_DIY => cpu_func!(ora_8, ora_16, diy),
            ORA_DILY => cpu_func!(ora_8, ora_16, dily),
            ORA_SR => cpu_func!(ora_8, ora_16, sr),
            ORA_SRIY => cpu_func!(ora_8, ora_16, sriy),
            PEA => self.pea(),
            PEI => self.pei(),
            PER => self.per(),
            PHA => self.pha(),
            PHB => self.phb(),
            PHD => self.phd(),
            PHK => self.phk(),
            PHP => self.php(),
            PHX => self.phx(),
            PHY => self.phy(),
            PLA => self.pla(),
            PLB => self.plb(),
            PLD => self.pld(),
            PLP => self.plp(),
            PLX => self.plx(),
            PLY => self.ply(),
            REP_I => cpu_func!(rep_8, rep_16, i),
            ROL_ACC => cpu_func!(rol_8, rol_16, acc),
            ROL_A => cpu_func!(rol_8, rol_16, a),
            ROL_D => cpu_func!(rol_8, rol_16, d),
            ROL_AX => cpu_func!(rol_8, rol_16, ax),
            ROL_DX => cpu_func!(rol_8, rol_16, dx),
            ROR_ACC => cpu_func!(ror_8, ror_16, acc),
            ROR_A => cpu_func!(ror_8, ror_16, a),
            ROR_D => cpu_func!(ror_8, ror_16, d),
            ROR_AX => cpu_func!(ror_8, ror_16, ax),
            ROR_DX => cpu_func!(ror_8, ror_16, dx),
            RTI => self.rti(),
            RTL => self.rtl(),
            RTS => self.rts(),
            SBC_I => cpu_func!(sbc_8, sbc_16, i),
            SBC_A => cpu_func!(sbc_8, sbc_16, a),
            SBC_AL => cpu_func!(sbc_8, sbc_16, al),
            SBC_D => cpu_func!(sbc_8, sbc_16, d),
            SBC_DI => cpu_func!(sbc_8, sbc_16, di),
            SBC_DIL => cpu_func!(sbc_8, sbc_16, dil),
            SBC_AX => cpu_func!(sbc_8, sbc_16, ax),
            SBC_ALX => cpu_func!(sbc_8, sbc_16, alx),
            SBC_AY => cpu_func!(sbc_8, sbc_16, ay),
            SBC_DX => cpu_func!(sbc_8, sbc_16, dx),
            SBC_DIX => cpu_func!(sbc_8, sbc_16, dix),
            SBC_DIY => cpu_func!(sbc_8, sbc_16, diy),
            SBC_DILY => cpu_func!(sbc_8, sbc_16, dily),
            SBC_SR => cpu_func!(sbc_8, sbc_16, sr),
            SBC_SRIY => cpu_func!(sbc_8, sbc_16, sriy),
            SEC => self.sec(),
            SED => self.sed(),
            SEI => self.sei(),
            SEP_I => cpu_func!(sep_8, sep_16, i),
            STA_A => cpu_func!(sta_8, sta_16, a),
            STA_AL => cpu_func!(sta_8, sta_16, al),
            STA_D => cpu_func!(sta_8, sta_16, d),
            STA_DI => cpu_func!(sta_8, sta_16, di),
            STA_DIL => cpu_func!(sta_8, sta_16, dil),
            STA_AX => cpu_func!(sta_8, sta_16, ax),
            STA_ALX => cpu_func!(sta_8, sta_16, alx),
            STA_AY => cpu_func!(sta_8, sta_16, ay),
            STA_DX => cpu_func!(sta_8, sta_16, dx),
            STA_DIX => cpu_func!(sta_8, sta_16, dix),
            STA_DIY => cpu_func!(sta_8, sta_16, diy),
            STA_DILY => cpu_func!(sta_8, sta_16, dily),
            STA_SR => cpu_func!(sta_8, sta_16, sr),
            STA_SRIY => cpu_func!(sta_8, sta_16, sriy),
            STP => self.stp(),
            STX_A => cpu_func!(stx_8, stx_16, a),
            STX_D => cpu_func!(stx_8, stx_16, d),
            STX_DY => cpu_func!(stx_8, stx_16, dy),
            STY_A => cpu_func!(sty_8, sty_16, a),
            STY_D => cpu_func!(sty_8, sty_16, d),
            STY_DX => cpu_func!(sty_8, sty_16, dx),
            STZ_A => cpu_func!(stz_8, stz_16, a),
            STZ_D => cpu_func!(stz_8, stz_16, d),
            STZ_AX => cpu_func!(stz_8, stz_16, ax),
            STZ_DX => cpu_func!(stz_8, stz_16, dx),
            TAX => self.tax(),
            TAY => self.tay(),
            TCD => self.tcd(),
            TCS => self.tcs(),
            TDC => self.tdc(),
            TRB_A => cpu_func!(trb_8, trb_16, a),
            TRB_D => cpu_func!(trb_8, trb_16, d),
            TSB_A => cpu_func!(tsb_8, tsb_16, a),
            TSB_D => cpu_func!(tsb_8, tsb_16, d),
            TSC => self.tsc(),
            TSX => self.tsx(),
            TXA => self.txa(),
            TXS => self.txs(),
            TXY => self.txy(),
            TYA => self.tya(),
            TYX => self.tyx(),
            WAI => self.wai(),
            WDM => self.wdm(),
            XBA => self.xba(),
            XCE => self.xce(),*/
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
}
