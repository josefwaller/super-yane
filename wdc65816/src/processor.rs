use crate::opcodes::*;
use crate::status_register::StatusRegister;
use crate::u24::u24;
use log::*;

use std::default::Default;
use std::fmt::Debug;

pub trait HasAddressBus {
    /// Read a single byte from memory
    fn read(&mut self, address: usize) -> u8;
    /// Write a single byte to memory
    fn write(&mut self, address: usize, value: u8);
    /// "Handle" an IO cycle
    fn io(&mut self);
}

#[derive(Clone)]
pub struct Processor {
    /// Program Counter
    pub pc: u16,
    /// Program Bank Register
    pub pbr: u8,
    /// Lower byte of the accumulator
    pub a: u8,
    /// Upper byte of the accumulator
    pub b: u8,
    /// X Register low
    pub xl: u8,
    /// X Register high
    pub xh: u8,
    /// Y Register low
    pub yl: u8,
    /// Y Register high
    pub yh: u8,
    /// Status Register
    pub p: StatusRegister,
    /// Direct Register
    pub d: u16,
    /// Data Bank Register
    pub dbr: u8,
    /// Stack Pointer
    pub s: u16,
}

impl Default for Processor {
    fn default() -> Self {
        Processor {
            pc: 0,
            pbr: 0,
            a: 0,
            b: 0,
            xl: 0,
            xh: 0,
            yl: 0,
            yh: 0,
            p: StatusRegister::default(),
            d: 0,
            dbr: 0,
            s: 0x01FF,
        }
    }
}

impl Debug for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC={:2X},C={:2X},X={:2X},Y={:2X},P={:X},D={:2X},DBR={:X},S={:2X}",
            self.pc,
            self.c(),
            self.x(),
            self.y(),
            self.p.to_byte(),
            self.d,
            self.dbr,
            self.s
        )
    }
}

/// Read a single byte from a memory given an 8-bit bank and a 16-bit address
fn read_u8(memory: &mut impl HasAddressBus, addr: u24) -> u8 {
    // Combine bank and address to form final address
    memory.read(addr.into())
}

fn read_u16<T: HasAddressBus>(memory: &mut T, addr: u24) -> u16 {
    // Read low first
    let low = read_u8(memory, addr);
    let high = read_u8(memory, addr.wrapping_add(1u32));
    u16::from_le_bytes([low, high])
}
fn read_u24(memory: &mut impl HasAddressBus, addr: u24) -> u24 {
    let low = read_u16(memory, addr);
    let high = read_u8(memory, addr.wrapping_add(2u32));
    u24::from(high, low)
}

/// Decrement a 16-bit number given it as two bytes in LE format
fn dec_16(low: u8, high: u8) -> [u8; 2] {
    let (low, carry) = low.overflowing_sub(1);
    [low, high.wrapping_sub(carry.into())]
}
/// Increment a 16-bit number given it as two bytes in LE format
fn inc_16(low: u8, high: u8) -> [u8; 2] {
    let (low, carry) = low.overflowing_add(1);
    [low, high.wrapping_add(carry.into())]
}

impl Processor {
    pub fn new() -> Self {
        Processor::default()
    }

    pub fn c(&self) -> u16 {
        if self.p.a_is_16bit() {
            self.c_true()
        } else {
            self.a as u16
        }
    }
    /// The "true" value of the C register, i.e. including the high bit regardless of
    /// the value of the m bit.
    pub fn c_true(&self) -> u16 {
        self.b as u16 * 0x100 + self.a as u16
    }
    /// Get the X register as a u16
    /// If the X register is 8-bit, only the bottom 8-bits will be used
    pub fn x(&self) -> u16 {
        if self.p.xy_is_16bit() {
            self.xh as u16 * 0x100 + self.xl as u16
        } else {
            self.xl as u16
        }
    }
    /// Get the Y register as a u16
    /// If the Y register is 8-bit, only the bottom 8-bits will be used
    pub fn y(&self) -> u16 {
        if self.p.xy_is_16bit() {
            self.yh as u16 * 0x100 + self.yl as u16
        } else {
            self.yl as u16
        }
    }
    /// Force the XH and YH registers to 0x00 if the status register's xb flags are set
    fn force_registers(&mut self) {
        if self.p.xy_is_8bit() {
            self.xh = 0;
            self.yh = 0;
        }
        if self.p.e {
            self.s = self.s & 0x01FF;
        }
    }
    /// Push a single byte to stack
    fn push_u8(&mut self, value: u8, memory: &mut impl HasAddressBus) {
        memory.write(self.s.into(), value);
        self.s = self.s.wrapping_sub(1);
        // Force high to 0 in emulation mode
        if self.p.e {
            self.s = self.s & 0x1FF;
        }
    }
    fn pull_u8(&mut self, memory: &mut impl HasAddressBus) -> u8 {
        self.s = self.s.wrapping_add(1);
        if self.p.e {
            self.s = self.s & 0x1FF;
        }
        memory.read(self.s.into())
    }
    // Push a u16 as two bytes in LE format
    fn push_u16_le(&mut self, [low, high]: [u8; 2], memory: &mut impl HasAddressBus) {
        self.push_u8(high, memory);
        self.push_u8(low, memory);
    }
    // Pull a u16 as two bytes in LE format
    fn pull_u16_le(&mut self, memory: &mut impl HasAddressBus) -> [u8; 2] {
        let low = self.pull_u8(memory);
        let high = self.pull_u8(memory);
        [low, high]
    }
    fn pull_u16(&mut self, memory: &mut impl HasAddressBus) -> u16 {
        u16::from_le_bytes(self.pull_u16_le(memory))
    }
    /// Push a 16-bit value to the stack
    fn push_u16(&mut self, value: u16, memory: &mut impl HasAddressBus) {
        self.push_u16_le(value.to_le_bytes(), memory);
    }
    fn adc(a: u8, b: u8, c: bool, d: bool) -> (u8, bool) {
        if d {
            let mut low = (a & 0xF) + (b & 0xF) + u8::from(c);
            if low > 0x09 {
                low += 0x06;
            }
            let mut res = (a as u16 & 0xF0) + (b as u16 & 0xF0) + low as u16;
            if res > 0x9F {
                res += 0x60;
            }
            ((res & 0xFF) as u8, res > 0x9F)
        } else {
            let (a, c2) = a.overflowing_add(b);
            let (a, c3) = a.overflowing_add(c.into());
            (a, c2 || c3)
        }
    }
    /// Add with Carry 8-bit
    fn adc_8(&mut self, value: u8) {
        let (a, c) = Processor::adc(self.a, value, self.p.c, self.p.d);
        self.p.v = ((self.a ^ a as u8) & (value ^ a)) & 0x80 != 0;
        self.p.n = (a & 0x80) != 0;
        self.p.z = a == 0;
        self.p.c = c;
        self.a = a;
    }
    /// Add with Carry 16-bit
    fn adc_16(&mut self, low: u8, high: u8) {
        let (a, c) = Processor::adc(self.a, low, self.p.c, self.p.d);
        self.a = a;
        let (b, c2) = Processor::adc(self.b, high, c, self.p.d);
        self.p.n = (b & 0x80) != 0;
        self.p.z = a == 0 && b == 0;
        self.p.c = c2;
        self.p.v = ((self.b ^ b as u8) & (high ^ b)) & 0x80 != 0;
        self.a = a;
        self.b = b;
    }
    fn sbc_d(&self, a: u8, b: u8, c: bool) -> (u8, bool) {
        // Note carry (c) is inverted here (0 = borrow, 1 = no borrow)
        let (low, c) = if (a & 0xF) >= (b & 0xF) + u8::from(!c) {
            ((a & 0x0F) - (b & 0x0F) - u8::from(!c), true)
        } else {
            (0xA + (a & 0xF) - (b & 0xF) - u8::from(!c), false)
        };
        let (high, c) = if (a & 0xF0) >= (b & 0xF0) + 0x10 * u8::from(!c) {
            ((a & 0xF0) - (b & 0xF0) - 0x10 * u8::from(!c), true)
        } else {
            (
                0xA0u8
                    .wrapping_add(a & 0xF0)
                    .wrapping_sub(b & 0xF0)
                    .wrapping_sub(0x10 * u8::from(!c)),
                false,
            )
        };
        ((high & 0xF0) | (low & 0x0F), c)
    }
    /// SuBtract with Carry (SBC) 8-bit
    fn sbc_8(&mut self, value: u8) {
        // Temporarily store some values so we can use the binary adc to set some flags even if we are in decimal mode
        let (a, c, d) = (self.a, self.p.c, self.p.d);
        self.p.d = false;
        self.adc_8(value ^ 0xFF);
        if d {
            (self.a, _) = self.sbc_d(a, value, c);
            self.p.n = self.a > 0x7F;
            self.p.z = self.a == 0;
            self.p.d = true;
        }
    }
    /// SuBtract with Carry (SBC) 16-bit
    fn sbc_16(&mut self, low: u8, high: u8) {
        // Store some values for the same reason as in sbc_8
        let (a, b, c, d) = (self.a, self.b, self.p.c, self.p.d);
        self.p.d = false;
        self.adc_16(low ^ 0xFF, high ^ 0xFF);
        // Overwrite result and some flags if in decimal mode
        if d {
            self.p.d = true;
            let (low, c) = self.sbc_d(a, low, c);
            let (high, _) = self.sbc_d(b, high, c);
            self.a = low;
            self.b = high;
            self.p.n = self.b > 0x7F;
            self.p.z = self.a == 0 && self.b == 0;
        }
    }
    /// Set common flags for AND, EOR, and ORA 8-bit
    fn bitwise_flags_8(&mut self) {
        self.p.n = self.a > 0x7F;
        self.p.z = self.a == 0;
    }
    /// Set common flags for AND, EOR, and ORA 16-bit
    fn bitwise_flags_16(&mut self) {
        self.p.n = self.b > 0x7F;
        self.p.z = self.a == 0 && self.b == 0;
    }
    /// And 8-bit
    fn and_8(&mut self, value: u8) {
        self.a = self.a & value;
        self.bitwise_flags_8();
    }
    /// And 16-bit
    fn and_16(&mut self, low: u8, high: u8) {
        self.a = self.a & low;
        self.b = self.b & high;
        self.bitwise_flags_16();
    }
    /// Exclusive Or (EOR) 8-bit
    fn eor_8(&mut self, value: u8) {
        self.a = self.a ^ value;
        self.bitwise_flags_8();
    }
    /// Exclusive OR (EOR) 16-bit
    fn eor_16(&mut self, low: u8, high: u8) {
        self.a = self.a ^ low;
        self.b = self.b ^ high;
        self.bitwise_flags_16();
    }
    /// OR with A (ORA) 8-bit
    fn ora_8(&mut self, value: u8) {
        self.a = self.a | value;
        self.bitwise_flags_8();
    }
    /// OR with A (ORA) 16-bit
    fn ora_16(&mut self, low: u8, high: u8) {
        self.a = self.a | low;
        self.b = self.b | high;
        self.bitwise_flags_16();
    }
    /// Set the flags after a 8-bit shift or rotate function
    fn shift_rotate_flags_8(&mut self, value: u8) {
        self.p.n = value > 0x7F;
        self.p.z = value == 0;
    }
    /// Set the flags after a 16-bit shift or rotate function
    fn shift_rotate_flags_16(&mut self, low: u8, high: u8) {
        self.p.n = high > 0x7F;
        self.p.z = (high == 0) && (low == 0);
    }
    /// ASL 8-bit
    fn asl_8(&mut self, value: u8) -> u8 {
        let value = value.rotate_left(1);
        self.p.c = (value & 0x01) != 0;
        self.shift_rotate_flags_8(value & 0xFE);
        value & 0xFE
    }
    /// ASL 16-bit
    fn asl_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let val = (high as u32 * 0x100 + low as u32) << 1;
        self.p.c = (val & 0x10000) != 0;
        let (low, high) = ((val & 0xFF) as u8, ((val & 0xFF00) >> 8) as u8);
        self.shift_rotate_flags_16(low, high);
        (low, high)
    }
    /// Logical Shift Right (LSR) 8-bit
    fn lsr_8(&mut self, value: u8) -> u8 {
        self.p.c = (value & 0x01) != 0;
        let value = value >> 1;
        self.shift_rotate_flags_8(value);
        value
    }
    /// Logical Shift Right (LSR) 16-bit
    fn lsr_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        self.p.c = (low & 0x01) != 0;
        let low = (low >> 1) + 0x80 * (high & 0x01);
        let high = high >> 1;
        self.shift_rotate_flags_16(low, high);
        (low, high)
    }
    /// Rotate Left (ROL) 8-bit
    fn rol_8(&mut self, value: u8) -> u8 {
        let c = (value & 0x80) != 0;
        let value = (value << 1) + u8::from(self.p.c);
        self.p.c = c;
        self.shift_rotate_flags_8(value);
        value
    }
    /// Rotate Left (ROL) 16-bit
    fn rol_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        // Low then high
        let low = self.rol_8(low);
        let high = self.rol_8(high);
        self.shift_rotate_flags_16(low, high);
        (low, high)
    }
    /// Rotate Right (ROR) 8-bit
    fn ror_8(&mut self, value: u8) -> u8 {
        let c = (value & 0x01) != 0;
        let value = (value >> 1) + 0x80 * u8::from(self.p.c);
        self.p.c = c;
        self.shift_rotate_flags_8(value);
        value
    }
    fn ror_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        // High then low
        let high = self.ror_8(high);
        let low = self.ror_8(low);
        self.shift_rotate_flags_16(low, high);
        (low, high)
    }

    /// Branch
    fn branch(&mut self, memory: &mut impl HasAddressBus, offset: u16) {
        memory.io();
        // self.pc = ((self.pc as i32).wrapping_add(offset as i32) % u16::MAX as i32) as u16;
        self.pc = self.pc.wrapping_add(offset);
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
    fn break_to(&mut self, memory: &mut impl HasAddressBus, addr_n: u16, addr_e: u16, set_b: bool) {
        if self.p.e {
            // Since we already incremented the PC by 1 we want to just add 1
            self.push_u16(self.pc.wrapping_add(1), memory);
            // Clone processor register to set B flag
            let mut p = self.p.clone();
            if set_b {
                p.xb = true;
            }
            self.push_u8(p.to_byte(), memory);
            self.pbr = 0x00;
            self.pc = read_u16(memory, u24::from(0, addr_e));
        } else {
            self.push_u8(self.pbr, memory);
            self.push_u16(self.pc.wrapping_add(1), memory);
            self.push_u8(self.p.to_byte(), memory);
            self.pbr = 0x00;
            self.pc = read_u16(memory, u24::from(0, addr_n));
        }
        self.p.d = false;
        self.p.i = true;
    }
    /// Decrement (DEC) 8-bit
    fn dec_8(&mut self, value: u8) -> u8 {
        let r = value.wrapping_sub(1);
        self.p.n = (r & 0x80) != 0;
        self.p.z = r == 0;
        r
    }
    /// Decrement (DEC) 16-bit
    fn dec_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let (low, carry) = low.overflowing_sub(1);
        let high = high.wrapping_sub(carry.into());
        self.p.n = (high & 0x80) != 0;
        self.p.z = (high == 0) && (low == 0);
        (low, high)
    }
    /// Increment (INC) 8-bit
    fn inc_8(&mut self, value: u8) -> u8 {
        let r = value.wrapping_add(1);
        self.p.n = (r & 0x80) != 0;
        self.p.z = r == 0;
        r
    }
    /// Increment (INC) 16-bit
    fn inc_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let (low, carry) = low.overflowing_add(1);
        let high = high.wrapping_add(carry.into());
        self.p.n = (high & 0x80) != 0;
        self.p.z = (high == 0) && (low == 0);
        (low, high)
    }
    /// Jump (JMP)
    fn jmp(&mut self, bank: u8, addr: u16) {
        self.pbr = bank;
        self.pc = addr;
    }
    /// Jump and Save Return/Jump to SubRoutine (JSR)
    fn jsr(&mut self, memory: &mut impl HasAddressBus, bank: u8, addr: u16) {
        self.push_u16(self.pc.wrapping_add(1), memory);
        self.jmp(bank, addr)
    }
    /// Return from interrupt
    fn rti(&mut self, memory: &mut impl HasAddressBus) {
        self.p = StatusRegister::from_byte(self.pull_u8(memory), self.p.e);
        self.force_registers();
        self.pc = self.pull_u16(memory);
        if !self.p.e {
            self.pbr = self.pull_u8(memory);
        }
    }
    /// ReTurn from Subroutine (RTS)
    fn rts(&mut self, memory: &mut impl HasAddressBus) {
        self.pc = self.pull_u16(memory).wrapping_add(1);
    }
    /// ReTurn from subroutine Long (RTL)
    fn rtl(&mut self, memory: &mut impl HasAddressBus) {
        self.pc = self.pull_u16(memory).wrapping_add(1);
        self.pbr = self.pull_u8(memory);
    }
    /// Set the load flags after loading an 8-bit value
    fn set_load_flags_8(&mut self, value: u8) {
        self.p.n = (value & 0x80) != 0;
        self.p.z = value == 0;
    }
    /// Set the load flags after loading a 16-bit value
    fn set_load_flags_16(&mut self, low: u8, high: u8) {
        self.p.n = (high & 0x80) != 0;
        self.p.z = (low == 0) && (high == 0);
    }
    /// LoaD into A (LDA) 8-bit
    fn lda_8(&mut self, value: u8) {
        self.a = value;
        self.set_load_flags_8(value);
    }
    /// LoaD into A (LDA) 16-bit
    fn lda_16(&mut self, low: u8, high: u8) {
        self.a = low;
        self.b = high;
        self.set_load_flags_16(low, high);
    }
    /// LoaD into X (LDX) 8-bit
    fn ldx_8(&mut self, value: u8) {
        self.xl = value;
        self.set_load_flags_8(value);
    }
    /// LoaD into X (LDX) 8-bit
    fn ldx_16(&mut self, low: u8, high: u8) {
        self.xl = low;
        self.xh = high;
        self.set_load_flags_16(low, high);
    }
    /// LoaD into Y (LDY) 8-bit
    fn ldy_8(&mut self, value: u8) {
        self.yl = value;
        self.set_load_flags_8(value);
    }
    /// LoaD into Y (LDY) 16-bit
    fn ldy_16(&mut self, low: u8, high: u8) {
        self.yl = low;
        self.yh = high;
        self.set_load_flags_16(low, high);
    }
    /// STore A (STA) 8-bit
    fn sta_8(&self) -> u8 {
        self.a
    }
    /// STore A (STA) 16-bit
    fn sta_16(&self) -> (u8, u8) {
        (self.a, self.b)
    }
    /// STore X (STX) 8-bit
    fn stx_8(&self) -> u8 {
        self.xl
    }
    /// STore X (STX) 16-bit
    fn stx_16(&self) -> (u8, u8) {
        (self.xl, self.xh)
    }
    /// STore Y (STY) 8-bit
    fn sty_8(&self) -> u8 {
        self.yl
    }
    /// STore Y (STY) 16-bit
    fn sty_16(&self) -> (u8, u8) {
        (self.yl, self.yh)
    }
    /// STore Zero (STZ) 8-bit
    fn stz_8(&self) -> u8 {
        0
    }
    /// STore Z (STZ) 16-bit
    fn stz_16(&self) -> (u8, u8) {
        (0, 0)
    }
    /// REset Processor status bits (REP)
    fn rep(&mut self, value: u8) {
        self.p = StatusRegister::from_byte(!value & self.p.to_byte(), self.p.e);
        self.force_registers();
    }
    /// SEt Processor status bits (SEP)
    fn sep(&mut self, value: u8) {
        self.p = StatusRegister::from_byte(value | self.p.to_byte(), self.p.e);
        self.force_registers();
    }
    /// Test and Reset Bits (TRB) 8-bit
    fn trb_8(&mut self, value: u8) -> u8 {
        self.p.z = (self.a & value) == 0;
        !self.a & value
    }
    /// Test and Reset Bits (TRB) 16-bit
    fn trb_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        self.p.z = ((self.a & low) | (self.b & high)) == 0;
        (!self.a & low, !self.b & high)
    }
    /// Test and Set Bits (TSB) 8-bit
    fn tsb_8(&mut self, value: u8) -> u8 {
        self.p.z = (self.a & value) == 0;
        self.a | value
    }
    /// Test and Set Bits (TSB) 16-bit
    fn tsb_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        self.p.z = ((self.a & low) | (self.b & high)) == 0;
        (self.a | low, self.b | high)
    }

    /// Individual methods for each addressing mode
    /// Combined with a CPU function to execute an instruction
    /// All return (bank, address) which are combined to form the final address in the
    /// cpu function to form the final 24-bit address

    /// Immediate addressing
    fn i(&mut self, _memory: &mut impl HasAddressBus) -> u24 {
        // Address is simply the next byte in the instruction
        let addr = u24::from(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    /// Absolute addressing
    fn a(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        // Read the 16 bit address off the instruction
        let addr = read_u16(memory, u24::from(self.pbr, self.pc));
        self.pc = self.pc.wrapping_add(2);
        u24::from(self.dbr, addr)
    }
    // Utility offset function for absolute indexed function
    fn a_off(&mut self, memory: &mut impl HasAddressBus, register: u16) -> u24 {
        let addr = read_u16(memory, u24::from(self.pbr, self.pc));
        memory.io();
        self.pc = self.pc.wrapping_add(2);
        u24::from(self.dbr, addr).wrapping_add(register)
    }
    /// Absolute X Indexed addressing
    fn ax(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        self.a_off(memory, self.x())
    }
    /// Absolute Y Indexed addressing
    fn ay(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        self.a_off(memory, self.y())
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = u24::from(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(3);
        read_u24(memory, addr)
    }
    /// Absolute Long X Indexed
    fn alx(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        self.al(memory).wrapping_add(self.x())
    }
    /// Direct addressing
    fn d(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let offset = read_u8(memory, u24::from(self.pbr, self.pc));
        // Extra cycle if direct register low is not 0
        if self.d & 0xFF != 0 {
            memory.io();
        }
        self.pc = self.pc.wrapping_add(1);
        u24::from(0x00, self.d.wrapping_add(offset as u16))
    }
    // Direct addressing with offset
    fn d_off(&mut self, memory: &mut impl HasAddressBus, register: u16) -> u24 {
        let addr = self.d(memory);
        memory.io();
        addr.wrapping_add(register).with_bank(0x00)
    }
    /// Direct X Indexed addressing
    fn dx(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        self.d_off(memory, self.x())
    }
    /// Direct Y Indexed addressing
    fn dy(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        self.d_off(memory, self.y())
    }
    /// Direct Indirect addressing
    fn di(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = self.d(memory);
        read_u24(memory, addr)
    }
    /// Direct Indirect X Indexed addressing
    fn dix(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = u24::from(
            0x00,
            self.d
                .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16)
                .wrapping_add(self.x()),
        );
        let addr = read_u16(memory, addr);
        memory.io();
        self.pc = self.pc.wrapping_add(1);
        u24::from(self.dbr, addr)
    }
    /// Direct Indirect Y Indexed addressing
    fn diy(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = u24::from(
            0x00,
            self.d
                .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16),
        );
        let addr = u24::from(self.dbr, read_u16(memory, addr));
        memory.io();
        self.pc = self.pc.wrapping_add(1);
        addr.wrapping_add(self.y())
    }
    /// Direct Indirect Long addressing
    fn dil(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = self
            .d
            .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16);
        self.pc = self.pc.wrapping_add(1);
        // Read the value of the pointer from memory
        read_u24(memory, u24::from(0x00, addr))
    }
    /// Direct Indirect Long Y Indexed addressing
    fn dily(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr: u24 = self.dil(memory).into();
        addr.wrapping_add(self.y())
    }
    /// Stack Relative addressing
    fn sr(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = self
            .s
            .wrapping_add(read_u8(memory, u24::from(self.pbr, self.pc)) as u16);
        memory.io();
        self.pc = self.pc.wrapping_add(1);
        u24::from(0x0, addr)
    }
    /// Stack Relative Indirect Y Indexed addressing
    fn sriy(&mut self, memory: &mut impl HasAddressBus) -> u24 {
        let addr = self.sr(memory);
        let addr = read_u16(memory, addr);
        u24::from(self.dbr, addr).wrapping_add(self.y())
    }

    /// Execute the next instruction in the program
    ///
    /// Read from the memory at the program counter to get the opcode,
    /// decode it, and execute it.
    /// Update the program counter accordingly.
    pub fn step<T: HasAddressBus>(&mut self, memory: &mut T) {
        macro_rules! read_func {
            ($f_8: ident, $f_16: ident, $addr: ident, $flag_8: ident) => {{
                let addr = self.$addr(memory);
                if self.p.$flag_8() {
                    self.$f_8(memory.read(addr.into()));
                } else {
                    self.$f_16(
                        memory.read(addr.into()),
                        memory.read(addr.wrapping_add(1u32).into()),
                    );
                }
            }};
        }
        // For Immediate addressing, we need to adjust how much we add to the PC depending on the register mode (8-bit or 6-bit)
        macro_rules! read_func_i {
            ($f_8: ident, $f_16: ident, $flag: ident) => {{
                read_func!($f_8, $f_16, i, $flag);
                if !self.p.$flag() {
                    self.pc = self.pc.wrapping_add(1);
                }
            }};
        }
        macro_rules! read_func_8 {
            ($f_8: ident, $addr: ident) => {{
                let addr = self.$addr(memory);
                self.$f_8(memory.read(addr.into()));
            }};
        }
        macro_rules! write_func {
            ($func_8: ident, $func_16: ident, $addr: ident, $flag: ident) => {{
                let addr = self.$addr(memory);
                if self.p.$flag() {
                    let value = self.$func_8();
                    memory.write(addr.into(), value);
                } else {
                    let (low, high) = self.$func_16();
                    memory.write(addr.into(), low);
                    memory.write(addr.wrapping_add(1u32).into(), high);
                }
            }};
        }
        macro_rules! read_write_func {
            ($func_8: ident, $func_16: ident, $get_addr: ident, $flag: ident) => {{
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
                let offset = read_u8(memory, u24::from(self.pbr, self.pc)) as i8;
                self.pc = self.pc.wrapping_add(1);
                if self.p.$flag == $value {
                    self.branch(memory, (offset as i16) as u16);
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
        macro_rules! acc_func {
            ($f_8: ident, $f_16: ident) => {
                reg_func!(a, b, a_is_16bit, $f_8, $f_16)
            };
        }
        macro_rules! x_func {
            ($f_8: ident, $f_16: ident) => {
                reg_func!(xl, xh, xy_is_16bit, $f_8, $f_16)
            };
        }
        macro_rules! y_func {
            ($f_8: ident, $f_16: ident) => {
                reg_func!(yl, yh, xy_is_16bit, $f_8, $f_16)
            };
        }
        macro_rules! push_reg {
            // Always 8-bit
            ($r: expr) => {{
                self.push_u8($r, memory);
            }};
            // Variable length
            ($rl: ident, $rh: ident, $flag_16: ident) => {{
                if self.p.$flag_16() {
                    self.push_u16_le([self.$rl, self.$rh], memory);
                } else {
                    self.push_u8(self.$rl, memory);
                }
            }};
        }
        macro_rules! pull_reg {
            // Always 8-bit
            ($r: expr) => {{
                $r = self.pull_u8(memory);
                self.p.n = ($r & 0x80) != 0;
                self.p.z = $r == 0;
            }};
            // Variable length
            ($rl: ident, $rh: ident, $flag_16: ident) => {{
                if self.p.$flag_16() {
                    let [low, high] = self.pull_u16_le(memory);
                    self.$rl = low;
                    self.$rh = high;
                    self.p.n = (high & 0x80) != 0;
                    self.p.z = high == 0 && low == 0;
                } else {
                    self.$rl = self.pull_u8(memory);
                    self.p.n = (self.$rl & 0x80) != 0;
                    self.p.z = self.$rl == 0;
                }
            }};
        }
        macro_rules! trans_reg {
            // Source Low/High, dest low/high, flag
            // Always 16 bit mode
            ($sl: expr, $sh: expr, $dl: expr, $dh: expr) => {{
                $dl = $sl;
                $dh = $sh;
                self.p.n = $sh > 0x7F;
                self.p.z = ($sl | $sh) == 0;
                self.force_registers();
            }};
            // Source Low/High, dest low/high, flag
            ($sl: expr, $sh: expr, $dl: expr, $dh: expr, $flag: ident) => {{
                if self.p.$flag() {
                    trans_reg!($sl, $sh, $dl, $dh);
                } else {
                    $dl = $sl;
                    self.p.n = $sl > 0x7F;
                    self.p.z = $sl == 0;
                }
                self.force_registers();
            }};
            // Transfer 2 u8s into a u16
            ($le: expr, $r: ident) => {{
                self.$r = u16::from_le_bytes($le);
                self.p.n = self.$r > 0x7FFF;
                self.p.z = self.$r == 0;
                self.force_registers();
            }};
            // Transfer a u16 into 2 u8s
            ($r: ident, $le: expr) => {{
                $le = self.$r.to_le_bytes();
                self.p.n = self.$r > 0x7FFF;
                self.p.z = self.$r == 0;
                self.force_registers();
            }};
        }
        macro_rules! block_func {
            ($xy_func: ident) => {{
                let dest_bank = read_u8(memory, u24::from(self.pbr, self.pc));
                self.dbr = dest_bank;
                let src_bank = read_u8(memory, u24::from(self.pbr, self.pc.wrapping_add(1)));
                let data = read_u8(memory, u24::from(src_bank, self.x()));
                memory.write(u24::from(dest_bank, self.y()).into(), data);
                [self.a, self.b] = dec_16(self.a, self.b);
                [self.xl, self.xh] = $xy_func(self.xl, self.xh);
                [self.yl, self.yh] = $xy_func(self.yl, self.yh);
                memory.io();
                memory.io();
                if self.a == 0xFF && self.b == 0xFF {
                    // Go to next instruction
                    self.pc = self.pc.wrapping_add(2);
                } else {
                    // Loop
                    self.pc = self.pc.wrapping_sub(1);
                }
            }};
        }
        let opcode = read_u8(memory, u24::from(self.pbr, self.pc));
        self.pc = self.pc.wrapping_add(1);

        match opcode {
            ADC_I => read_func_i!(adc_8, adc_16, a_is_8bit),
            ADC_A => read_func!(adc_8, adc_16, a, a_is_8bit),
            ADC_AX => read_func!(adc_8, adc_16, ax, a_is_8bit),
            ADC_AY => read_func!(adc_8, adc_16, ay, a_is_8bit),
            ADC_AL => read_func!(adc_8, adc_16, al, a_is_8bit),
            ADC_ALX => read_func!(adc_8, adc_16, alx, a_is_8bit),
            ADC_D => read_func!(adc_8, adc_16, d, a_is_8bit),
            ADC_DX => read_func!(adc_8, adc_16, dx, a_is_8bit),
            ADC_DI => read_func!(adc_8, adc_16, di, a_is_8bit),
            ADC_DIX => read_func!(adc_8, adc_16, dix, a_is_8bit),
            ADC_DIY => read_func!(adc_8, adc_16, diy, a_is_8bit),
            ADC_DIL => read_func!(adc_8, adc_16, dil, a_is_8bit),
            ADC_DILY => read_func!(adc_8, adc_16, dily, a_is_8bit),
            ADC_SR => read_func!(adc_8, adc_16, sr, a_is_8bit),
            ADC_SRIY => read_func!(adc_8, adc_16, sriy, a_is_8bit),
            AND_I => read_func_i!(and_8, and_16, a_is_8bit),
            AND_A => read_func!(and_8, and_16, a, a_is_8bit),
            AND_AL => read_func!(and_8, and_16, al, a_is_8bit),
            AND_D => read_func!(and_8, and_16, d, a_is_8bit),
            AND_DI => read_func!(and_8, and_16, di, a_is_8bit),
            AND_DIL => read_func!(and_8, and_16, dil, a_is_8bit),
            AND_AX => read_func!(and_8, and_16, ax, a_is_8bit),
            AND_ALX => read_func!(and_8, and_16, alx, a_is_8bit),
            AND_AY => read_func!(and_8, and_16, ay, a_is_8bit),
            AND_DX => read_func!(and_8, and_16, dx, a_is_8bit),
            AND_DIX => read_func!(and_8, and_16, dix, a_is_8bit),
            AND_DIY => read_func!(and_8, and_16, diy, a_is_8bit),
            AND_DILY => read_func!(and_8, and_16, dily, a_is_8bit),
            AND_SR => read_func!(and_8, and_16, sr, a_is_8bit),
            AND_SRIY => read_func!(and_8, and_16, sriy, a_is_8bit),
            ASL_ACC => acc_func!(asl_8, asl_16),
            ASL_A => read_write_func!(asl_8, asl_16, a, a_is_8bit),
            ASL_D => read_write_func!(asl_8, asl_16, d, a_is_8bit),
            ASL_AX => read_write_func!(asl_8, asl_16, ax, a_is_8bit),
            ASL_DX => read_write_func!(asl_8, asl_16, dx, a_is_8bit),
            BCC => branch_if!(c, false),
            BCS => branch_if!(c, true),
            BEQ => branch_if!(z, true),
            BIT_I => {
                // > Immediate addressing only affects the z flag (with the result of the bitwise And), but does not affect the n and v flags.
                // > All other addressing modes of BIT affect the n, v, and z flags.
                // > This is the only instruction in the 6502 family where the flags affected depends on the addressing mode.
                // http://www.6502.org/tutorials/65c816opcodes.html#6.1.2.2
                read_func_i!(bit_i_8, bit_i_16, a_is_8bit);
            }
            BIT_A => read_func!(bit_8, bit_16, a, a_is_8bit),
            BIT_D => read_func!(bit_8, bit_16, d, a_is_8bit),
            BIT_AX => read_func!(bit_8, bit_16, ax, a_is_8bit),
            BIT_DX => read_func!(bit_8, bit_16, dx, a_is_8bit),
            BMI => branch_if!(n, true),
            BNE => branch_if!(z, false),
            BPL => branch_if!(n, false),
            BRA => {
                let addr = (read_u8(memory, u24::from(self.pbr, self.pc)) as i8) as i16;
                self.pc = self.pc.wrapping_add(1);
                self.branch(memory, addr as u16);
            }
            BRK => self.break_to(memory, 0xFFE6, 0xFFFE, true),
            BRL => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc));
                self.pc = self.pc.wrapping_add(2);
                self.branch(memory, addr);
            }
            BVC => branch_if!(v, false),
            BVS => branch_if!(v, true),
            CLC => set_flag!(c, false),
            CLD => set_flag!(d, false),
            CLI => set_flag!(i, false),
            CLV => set_flag!(v, false),
            CMP_I => read_func_i!(cmp_8, cmp_16, a_is_8bit),
            CMP_A => read_func!(cmp_8, cmp_16, a, a_is_8bit),
            CMP_AL => read_func!(cmp_8, cmp_16, al, a_is_8bit),
            CMP_D => read_func!(cmp_8, cmp_16, d, a_is_8bit),
            CMP_DI => read_func!(cmp_8, cmp_16, di, a_is_8bit),
            CMP_DIL => read_func!(cmp_8, cmp_16, dil, a_is_8bit),
            CMP_AX => read_func!(cmp_8, cmp_16, ax, a_is_8bit),
            CMP_ALX => read_func!(cmp_8, cmp_16, alx, a_is_8bit),
            CMP_AY => read_func!(cmp_8, cmp_16, ay, a_is_8bit),
            CMP_DX => read_func!(cmp_8, cmp_16, dx, a_is_8bit),
            CMP_DIX => read_func!(cmp_8, cmp_16, dix, a_is_8bit),
            CMP_DIY => read_func!(cmp_8, cmp_16, diy, a_is_8bit),
            CMP_DILY => read_func!(cmp_8, cmp_16, dily, a_is_8bit),
            CMP_SR => read_func!(cmp_8, cmp_16, sr, a_is_8bit),
            CMP_SRIY => read_func!(cmp_8, cmp_16, sriy, a_is_8bit),
            COP => self.break_to(memory, 0xFFE4, 0xFFF4, false),
            CPX_I => read_func_i!(cpx_8, cpx_16, xy_is_8bit),
            CPX_A => read_func!(cpx_8, cpx_16, a, xy_is_8bit),
            CPX_D => read_func!(cpx_8, cpx_16, d, xy_is_8bit),
            CPY_I => read_func_i!(cpy_8, cpy_16, xy_is_8bit),
            CPY_A => read_func!(cpy_8, cpy_16, a, xy_is_8bit),
            CPY_D => read_func!(cpy_8, cpy_16, d, xy_is_8bit),
            DEC_ACC => acc_func!(dec_8, dec_16),
            DEC_A => read_write_func!(dec_8, dec_16, a, a_is_8bit),
            DEC_D => read_write_func!(dec_8, dec_16, d, a_is_8bit),
            DEC_AX => read_write_func!(dec_8, dec_16, ax, a_is_8bit),
            DEC_DX => read_write_func!(dec_8, dec_16, dx, a_is_8bit),
            DEX => x_func!(dec_8, dec_16),
            DEY => y_func!(dec_8, dec_16),
            EOR_I => read_func_i!(eor_8, eor_16, a_is_8bit),
            EOR_A => read_func!(eor_8, eor_16, a, a_is_8bit),
            EOR_AL => read_func!(eor_8, eor_16, al, a_is_8bit),
            EOR_D => read_func!(eor_8, eor_16, d, a_is_8bit),
            EOR_DI => read_func!(eor_8, eor_16, di, a_is_8bit),
            EOR_DIL => read_func!(eor_8, eor_16, dil, a_is_8bit),
            EOR_AX => read_func!(eor_8, eor_16, ax, a_is_8bit),
            EOR_ALX => read_func!(eor_8, eor_16, alx, a_is_8bit),
            EOR_AY => read_func!(eor_8, eor_16, ay, a_is_8bit),
            EOR_DX => read_func!(eor_8, eor_16, dx, a_is_8bit),
            EOR_DIX => read_func!(eor_8, eor_16, dix, a_is_8bit),
            EOR_DIY => read_func!(eor_8, eor_16, diy, a_is_8bit),
            EOR_DILY => read_func!(eor_8, eor_16, dily, a_is_8bit),
            EOR_SR => read_func!(eor_8, eor_16, sr, a_is_8bit),
            EOR_SRIY => read_func!(eor_8, eor_16, sriy, a_is_8bit),
            INC_ACC => acc_func!(inc_8, inc_16),
            INC_A => read_write_func!(inc_8, inc_16, a, a_is_8bit),
            INC_D => read_write_func!(inc_8, inc_16, d, a_is_8bit),
            INC_AX => read_write_func!(inc_8, inc_16, ax, a_is_8bit),
            INC_DX => read_write_func!(inc_8, inc_16, dx, a_is_8bit),
            INX => x_func!(inc_8, inc_16),
            INY => y_func!(inc_8, inc_16),
            // These all need to be custom, since JMP ABSOLUTE doesn't actually read the byte at the absolute address, but jumps to it
            JMP_A => self.jmp(self.pbr, read_u16(memory, u24::from(self.pbr, self.pc))),
            JMP_AI => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc));
                self.jmp(self.pbr, read_u16(memory, u24::from(0x00, addr)));
            }
            JMP_AIX => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc)).wrapping_add(self.x());
                self.jmp(self.pbr, read_u16(memory, u24::from(self.pbr, addr)));
            }
            JMP_AL => {
                self.jmp(
                    read_u8(memory, u24::from(self.pbr, self.pc.wrapping_add(2))),
                    read_u16(memory, u24::from(self.pbr, self.pc)),
                );
            }
            JMP_AIL => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc));
                self.jmp(
                    read_u8(memory, u24::from(0x00, addr).wrapping_add(2u32)),
                    read_u16(memory, u24::from(0x00, addr)),
                );
            }
            JSR_A => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc));
                self.jsr(memory, self.pbr, addr);
            }
            JSR_AIX => {
                let addr = read_u16(memory, u24::from(self.pbr, self.pc)).wrapping_add(self.x());
                let addr = read_u16(memory, u24::from(self.pbr, addr));
                self.jsr(memory, self.pbr, addr);
            }
            JSL => {
                self.push_u8(self.pbr, memory);
                self.push_u16(self.pc.wrapping_add(2), memory);
                let addr = read_u16(memory, u24::from(self.pbr, self.pc));
                let bank = read_u8(memory, u24::from(self.pbr, self.pc.wrapping_add(2)));
                self.jmp(bank, addr);
            }
            LDA_I => read_func_i!(lda_8, lda_16, a_is_8bit),
            LDA_A => read_func!(lda_8, lda_16, a, a_is_8bit),
            LDA_AL => read_func!(lda_8, lda_16, al, a_is_8bit),
            LDA_D => read_func!(lda_8, lda_16, d, a_is_8bit),
            LDA_DI => read_func!(lda_8, lda_16, di, a_is_8bit),
            LDA_DIL => read_func!(lda_8, lda_16, dil, a_is_8bit),
            LDA_AX => read_func!(lda_8, lda_16, ax, a_is_8bit),
            LDA_ALX => read_func!(lda_8, lda_16, alx, a_is_8bit),
            LDA_AY => read_func!(lda_8, lda_16, ay, a_is_8bit),
            LDA_DX => read_func!(lda_8, lda_16, dx, a_is_8bit),
            LDA_DIX => read_func!(lda_8, lda_16, dix, a_is_8bit),
            LDA_DIY => read_func!(lda_8, lda_16, diy, a_is_8bit),
            LDA_DILY => read_func!(lda_8, lda_16, dily, a_is_8bit),
            LDA_SR => read_func!(lda_8, lda_16, sr, a_is_8bit),
            LDA_SRIY => read_func!(lda_8, lda_16, sriy, a_is_8bit),
            LDX_I => read_func_i!(ldx_8, ldx_16, xy_is_8bit),
            LDX_A => read_func!(ldx_8, ldx_16, a, xy_is_8bit),
            LDX_D => read_func!(ldx_8, ldx_16, d, xy_is_8bit),
            LDX_AY => read_func!(ldx_8, ldx_16, ay, xy_is_8bit),
            LDX_DY => read_func!(ldx_8, ldx_16, dy, xy_is_8bit),
            LDY_I => read_func_i!(ldy_8, ldy_16, xy_is_8bit),
            LDY_A => read_func!(ldy_8, ldy_16, a, xy_is_8bit),
            LDY_D => read_func!(ldy_8, ldy_16, d, xy_is_8bit),
            LDY_AX => read_func!(ldy_8, ldy_16, ax, xy_is_8bit),
            LDY_DX => read_func!(ldy_8, ldy_16, dx, xy_is_8bit),
            LSR_ACC => acc_func!(lsr_8, lsr_16),
            LSR_A => read_write_func!(lsr_8, lsr_16, a, a_is_8bit),
            LSR_D => read_write_func!(lsr_8, lsr_16, d, a_is_8bit),
            LSR_AX => read_write_func!(lsr_8, lsr_16, ax, a_is_8bit),
            LSR_DX => read_write_func!(lsr_8, lsr_16, dx, a_is_8bit),
            MVN => block_func!(inc_16),
            MVP => block_func!(dec_16),
            NOP => memory.io(),
            ORA_I => read_func_i!(ora_8, ora_16, a_is_8bit),
            ORA_A => read_func!(ora_8, ora_16, a, a_is_8bit),
            ORA_AL => read_func!(ora_8, ora_16, al, a_is_8bit),
            ORA_D => read_func!(ora_8, ora_16, d, a_is_8bit),
            ORA_DI => read_func!(ora_8, ora_16, di, a_is_8bit),
            ORA_DIL => read_func!(ora_8, ora_16, dil, a_is_8bit),
            ORA_AX => read_func!(ora_8, ora_16, ax, a_is_8bit),
            ORA_ALX => read_func!(ora_8, ora_16, alx, a_is_8bit),
            ORA_AY => read_func!(ora_8, ora_16, ay, a_is_8bit),
            ORA_DX => read_func!(ora_8, ora_16, dx, a_is_8bit),
            ORA_DIX => read_func!(ora_8, ora_16, dix, a_is_8bit),
            ORA_DIY => read_func!(ora_8, ora_16, diy, a_is_8bit),
            ORA_DILY => read_func!(ora_8, ora_16, dily, a_is_8bit),
            ORA_SR => read_func!(ora_8, ora_16, sr, a_is_8bit),
            ORA_SRIY => read_func!(ora_8, ora_16, sriy, a_is_8bit),
            PEA => {
                // Push the opcode onto the stack
                let addr = self.i(memory);
                let value = read_u16(memory, addr);
                self.push_u16(value, memory);
                self.pc = self.pc.wrapping_add(1);
            }
            PEI => {
                let pointer = self.d(memory);
                self.push_u16(read_u16(memory, pointer), memory);
            }
            PER => {
                // Add operand to address of next instruction
                let value = self
                    .pc
                    .wrapping_add(2)
                    .wrapping_add(read_u16(memory, u24::from(self.pbr, self.pc)));
                self.push_u16(value, memory);
                self.pc = self.pc.wrapping_add(2);
            }
            PHA => push_reg!(a, b, a_is_16bit),
            PHB => push_reg!(self.dbr),
            // Custom for 16-bit value
            PHD => self.push_u16(self.d, memory),
            PHK => push_reg!(self.pbr),
            PHP => push_reg!(self.p.to_byte()),
            PHX => push_reg!(xl, xh, xy_is_16bit),
            PHY => push_reg!(yl, yh, xy_is_16bit),
            PLA => pull_reg!(a, b, a_is_16bit),
            PLB => {
                pull_reg!(self.dbr);
                self.p.n = self.dbr > 0x7F;
                self.p.z = self.dbr == 0;
            }
            PLD => {
                self.d = self.pull_u16(memory);
                self.p.n = self.d > 0x7FFF;
                self.p.z = self.d == 0;
            }
            PLP => {
                self.p = StatusRegister::from_byte(self.pull_u8(memory), self.p.e);
                self.force_registers();
            }
            PLX => pull_reg!(xl, xh, xy_is_16bit),
            PLY => pull_reg!(yl, yh, xy_is_16bit),
            REP_I => read_func_8!(rep, i),
            ROL_ACC => acc_func!(rol_8, rol_16),
            ROL_A => read_write_func!(rol_8, rol_16, a, a_is_8bit),
            ROL_D => read_write_func!(rol_8, rol_16, d, a_is_8bit),
            ROL_AX => read_write_func!(rol_8, rol_16, ax, a_is_8bit),
            ROL_DX => read_write_func!(rol_8, rol_16, dx, a_is_8bit),
            ROR_ACC => acc_func!(ror_8, ror_16),
            ROR_A => read_write_func!(ror_8, ror_16, a, a_is_8bit),
            ROR_D => read_write_func!(ror_8, ror_16, d, a_is_8bit),
            ROR_AX => read_write_func!(ror_8, ror_16, ax, a_is_8bit),
            ROR_DX => read_write_func!(ror_8, ror_16, dx, a_is_8bit),
            RTI => self.rti(memory),
            RTL => self.rtl(memory),
            RTS => self.rts(memory),
            SBC_I => read_func_i!(sbc_8, sbc_16, a_is_8bit),
            SBC_A => read_func!(sbc_8, sbc_16, a, a_is_8bit),
            SBC_AL => read_func!(sbc_8, sbc_16, al, a_is_8bit),
            SBC_D => read_func!(sbc_8, sbc_16, d, a_is_8bit),
            SBC_DI => read_func!(sbc_8, sbc_16, di, a_is_8bit),
            SBC_DIL => read_func!(sbc_8, sbc_16, dil, a_is_8bit),
            SBC_AX => read_func!(sbc_8, sbc_16, ax, a_is_8bit),
            SBC_ALX => read_func!(sbc_8, sbc_16, alx, a_is_8bit),
            SBC_AY => read_func!(sbc_8, sbc_16, ay, a_is_8bit),
            SBC_DX => read_func!(sbc_8, sbc_16, dx, a_is_8bit),
            SBC_DIX => read_func!(sbc_8, sbc_16, dix, a_is_8bit),
            SBC_DIY => read_func!(sbc_8, sbc_16, diy, a_is_8bit),
            SBC_DILY => read_func!(sbc_8, sbc_16, dily, a_is_8bit),
            SBC_SR => read_func!(sbc_8, sbc_16, sr, a_is_8bit),
            SBC_SRIY => read_func!(sbc_8, sbc_16, sriy, a_is_8bit),
            SEC => set_flag!(c, true),
            SED => set_flag!(d, true),
            SEI => set_flag!(i, true),
            SEP_I => read_func_8!(sep, i),
            STA_A => write_func!(sta_8, sta_16, a, a_is_8bit),
            STA_AL => write_func!(sta_8, sta_16, al, a_is_8bit),
            STA_D => write_func!(sta_8, sta_16, d, a_is_8bit),
            STA_DI => write_func!(sta_8, sta_16, di, a_is_8bit),
            STA_DIL => write_func!(sta_8, sta_16, dil, a_is_8bit),
            STA_AX => write_func!(sta_8, sta_16, ax, a_is_8bit),
            STA_ALX => write_func!(sta_8, sta_16, alx, a_is_8bit),
            STA_AY => write_func!(sta_8, sta_16, ay, a_is_8bit),
            STA_DX => write_func!(sta_8, sta_16, dx, a_is_8bit),
            STA_DIX => write_func!(sta_8, sta_16, dix, a_is_8bit),
            STA_DIY => write_func!(sta_8, sta_16, diy, a_is_8bit),
            STA_DILY => write_func!(sta_8, sta_16, dily, a_is_8bit),
            STA_SR => write_func!(sta_8, sta_16, sr, a_is_8bit),
            STA_SRIY => write_func!(sta_8, sta_16, sriy, a_is_8bit),
            STP => {
                // self.pc = self.pc.wrapping_sub(1);
            }
            STX_A => write_func!(stx_8, stx_16, a, xy_is_8bit),
            STX_D => write_func!(stx_8, stx_16, d, xy_is_8bit),
            STX_DY => write_func!(stx_8, stx_16, dy, xy_is_8bit),
            STY_A => write_func!(sty_8, sty_16, a, xy_is_8bit),
            STY_D => write_func!(sty_8, sty_16, d, xy_is_8bit),
            STY_DX => write_func!(sty_8, sty_16, dx, xy_is_8bit),
            STZ_A => write_func!(stz_8, stz_16, a, a_is_8bit),
            STZ_D => write_func!(stz_8, stz_16, d, a_is_8bit),
            STZ_AX => write_func!(stz_8, stz_16, ax, a_is_8bit),
            STZ_DX => write_func!(stz_8, stz_16, dx, a_is_8bit),
            TAX => trans_reg!(self.a, self.b, self.xl, self.xh, xy_is_16bit),
            TAY => trans_reg!(self.a, self.b, self.yl, self.yh, xy_is_16bit),
            TCD => trans_reg!([self.a, self.b], d),
            TCS => {
                // This one done manually since transferring to S does not set any flags
                self.s = u16::from_le_bytes([self.a, self.b]);
                self.force_registers();
            }
            TDC => trans_reg!(d, [self.a, self.b]),
            TRB_A => read_write_func!(trb_8, trb_16, a, a_is_16bit),
            TRB_D => read_write_func!(trb_8, trb_16, d, a_is_16bit),
            TSB_A => read_write_func!(tsb_8, tsb_16, a, a_is_16bit),
            TSB_D => read_write_func!(tsb_8, tsb_16, d, a_is_16bit),
            TSC => trans_reg!(s, [self.a, self.b]),
            TSX => {
                let [low, high] = self.s.to_le_bytes();
                trans_reg!(low, high, self.xl, self.xh, xy_is_16bit)
            }
            TXA => trans_reg!(self.xl, self.xh, self.a, self.b, a_is_16bit),
            TXS => {
                // This one dones manually since it does not set any flags
                self.s = u16::from_le_bytes([self.xl, self.xh]);
                self.force_registers();
            }
            TXY => trans_reg!(self.xl, self.xh, self.yl, self.yh, xy_is_16bit),
            TYA => trans_reg!(self.yl, self.yh, self.a, self.b, a_is_16bit),
            TYX => trans_reg!(self.yl, self.yh, self.xl, self.xh, xy_is_16bit),
            WAI => {
                debug!("WAI")
            }
            WDM => {
                // Read and ignore next byte
                read_u8(memory, u24::from(self.pbr, self.pc.wrapping_add(1)));
                self.pc = self.pc.wrapping_add(1)
            }
            XBA => {
                std::mem::swap(&mut self.a, &mut self.b);
                self.p.n = self.a > 0x7F;
                self.p.z = self.a == 0;
                memory.io();
                memory.io();
            }
            XCE => {
                std::mem::swap(&mut self.p.c, &mut self.p.e);
                self.force_registers();
            }
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
    pub fn reset(&mut self, rest: &mut impl HasAddressBus) {
        self.p.e = true;
        self.pbr = 0x00;
        self.pc = u16::from_le_bytes([rest.read(0xFFFC), rest.read(0xFFFD)]);
        self.dbr = 0;
        self.d = 0;
    }
}
