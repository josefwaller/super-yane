use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{ProgramStatusWord, opcodes::*};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Processor {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: ProgramStatusWord,
}

pub trait HasAddressBus {
    fn io(&mut self);
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
}

impl Display for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A={:02X} X={:02X} Y={:02X} SP={:02X} PC={:04X} PSW=[{}]",
            self.a, self.x, self.y, self.sp, self.pc, self.psw,
        )
    }
}

impl Processor {
    fn read_u16(&self, addr: usize, bus: &mut impl HasAddressBus) -> u16 {
        let low = bus.read(addr);
        let high = bus.read((addr as u16).wrapping_add(1) as usize);
        u16::from_le_bytes([low, high])
    }
    fn write_u16(&self, addr: usize, value: u16, bus: &mut impl HasAddressBus) {
        let [low, high] = value.to_le_bytes();
        bus.write(addr as usize, low);
        bus.write(addr.wrapping_add(1) as usize, high);
    }
    fn push_to_stack_u8(&mut self, val: u8, bus: &mut impl HasAddressBus) {
        bus.write(self.sp as usize + 0x100, val);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pull_from_stack_u8(&mut self, bus: &mut impl HasAddressBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let val = bus.read(self.sp as usize + 0x100);
        val
    }
    fn push_to_stack_u16(&mut self, val: u16, bus: &mut impl HasAddressBus) {
        let [l, h] = val.to_le_bytes();
        self.push_to_stack_u8(h, bus);
        self.push_to_stack_u8(l, bus);
    }
    fn pull_from_stack_u16(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        let l = self.pull_from_stack_u8(bus);
        let h = self.pull_from_stack_u8(bus);
        u16::from_le_bytes([l, h])
    }

    /// Set the N and Z flags based on the value provided
    fn set_nz(&mut self, v: u8) {
        self.psw.z = v == 0;
        self.psw.n = (v & 0x80) != 0;
    }
    fn set_nz_16le(&mut self, l: u8, h: u8) {
        self.psw.z = l == 0 && h == 0;
        self.psw.n = (h & 0x80) != 0;
    }
    /// Read the next value at the PC (and increment the PC) and branch if `value` is true.
    /// If `value` is false, just increment the PC.
    fn read_and_branch_if(&mut self, value: bool, bus: &mut impl HasAddressBus) {
        if value {
            self.branch_imm(bus);
        } else {
            self.pc = self.pc.wrapping_add(1);
        }
    }

    // FUNCTIONS USED IN MACROS
    // COMBINED WITH ONE OR MORE ADDRESSING MODE FUNCTIONS
    fn adc(&mut self, l: u8, r: u8) -> u8 {
        let (v, c1) = l.overflowing_add(r);
        let (v, c2) = v.overflowing_add(self.psw.c.into());
        self.psw.h = (l & 0x0F) + (r & 0x0F) + u8::from(self.psw.c) > 0x0F;
        self.psw.c = c1 | c2;
        self.psw.v = ((l ^ v) & (r ^ v)) & 0x80 != 0;
        self.set_nz(v);
        v
    }
    fn and(&mut self, l: u8, r: u8) -> u8 {
        let v = l & r;
        self.set_nz(v);
        v
    }
    fn asl(&mut self, v: u8) -> u8 {
        let v = v.rotate_left(1);
        self.set_nz(v & 0xFE);
        self.psw.c = (v & 0x01) != 0;
        v & 0xFE
    }
    fn branch_imm(&mut self, bus: &mut impl HasAddressBus) {
        let offset_addr = self.imm(bus);
        let offset = bus.read(offset_addr as usize);
        self.pc =
            ((self.pc as isize).wrapping_add((offset as i8) as isize) % u16::MAX as isize) as u16;
    }
    fn call(&mut self, addr: u16, bus: &mut impl HasAddressBus) {
        self.push_to_stack_u16(self.pc, bus);
        self.pc = addr;
    }
    fn cmp(&mut self, lhs: u8, rhs: u8) {
        let (r, c) = lhs.overflowing_sub(rhs);
        self.set_nz(r);
        self.psw.c = !c;
    }
    fn cmp_a(&mut self, v: u8) {
        self.cmp(self.a, v);
    }
    fn cmp_x(&mut self, v: u8) {
        self.cmp(self.x, v);
    }
    fn cmp_y(&mut self, v: u8) {
        self.cmp(self.y, v)
    }
    fn dec(&mut self, v: u8) -> u8 {
        let v = v.wrapping_sub(1);
        self.set_nz(v);
        v
    }
    fn eor(&mut self, a: u8, b: u8) -> u8 {
        let v = a ^ b;
        self.set_nz(v);
        v
    }
    fn inc(&mut self, v: u8) -> u8 {
        let v = v.wrapping_add(1);
        self.set_nz(v);
        v
    }
    fn lsr(&mut self, v: u8) -> u8 {
        let v = v.rotate_right(1);
        self.psw.c = (v & 0x80) != 0;
        let v = v & 0x7F;
        self.set_nz(v);
        v
    }
    fn mov(&mut self, v: u8) -> u8 {
        self.set_nz(v);
        v
    }
    // Move without setting flags
    fn mov_no_flags(&mut self, v: u8) -> u8 {
        v
    }
    fn or(&mut self, a: u8, b: u8) -> u8 {
        let v = a | b;
        self.set_nz(v);
        v
    }
    fn or_a(&mut self, v: u8) {
        self.a = self.or(self.a, v);
    }
    fn rol(&mut self, v: u8) -> u8 {
        let c = (v & 0x80) != 0;
        let v = (v << 1) + u8::from(self.psw.c);
        self.set_nz(v);
        self.psw.c = c;
        v
    }
    fn ror(&mut self, v: u8) -> u8 {
        let c = (v & 0x01) != 0;
        let v = (v >> 1) | (u8::from(self.psw.c) << 7);
        self.set_nz(v);
        self.psw.c = c;
        v
    }
    fn sbc(&mut self, lhs: u8, rhs: u8) -> u8 {
        self.adc(lhs, rhs ^ 0xFF)
    }
    fn tclr(&mut self, v: u8) -> u8 {
        // TCLR and TSET compute the subtraction, which is only used
        // to set the flags
        self.set_nz(self.a.wrapping_sub(v));
        let v = v & !self.a;
        v
    }
    fn tset(&mut self, v: u8) -> u8 {
        self.set_nz(self.a.wrapping_sub(v));
        let v = v | self.a;
        v
    }

    // ADDRESSING MODE FUNCTIONS
    // RETURNS THE ADDRESS OF THE VALUE, RATHER THAN THE VALUE ITSELF

    /// Immediate addressing
    fn imm(&mut self, _bus: &mut impl HasAddressBus) -> usize {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        pc as usize
    }
    /// Direct page addressing with offset
    fn d_off(&mut self, r: u8, bus: &mut impl HasAddressBus) -> usize {
        let addr = self.pc as usize;
        self.pc = self.pc.wrapping_add(1);
        let addr = bus.read(addr);
        0x100 * usize::from(self.psw.p) + addr.wrapping_add(r) as usize
    }
    /// Direct page addressing
    fn d(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.d_off(0, bus)
    }
    /// Direct page with X offset
    fn dx(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.d_off(self.x, bus)
    }
    /// Direct page with Y offset
    fn dy(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.d_off(self.y, bus)
    }
    /// Direct page word.
    /// Returns the address of the [low, high] bytes.
    /// May page wrap.
    fn dw(&mut self, bus: &mut impl HasAddressBus) -> [usize; 2] {
        let addr = bus.read(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        let off = 0x100 * usize::from(self.psw.p);
        [off + addr as usize, off + addr.wrapping_add(1) as usize]
    }
    fn ix(&mut self, _bus: &mut impl HasAddressBus) -> usize {
        self.x as usize
    }
    fn iy(&mut self, _bus: &mut impl HasAddressBus) -> usize {
        self.y as usize
    }
    fn idx(&mut self, bus: &mut impl HasAddressBus) -> usize {
        let addr = self.d_off(self.x, bus);
        self.read_u16(addr, bus) as usize
    }
    fn idy(&mut self, bus: &mut impl HasAddressBus) -> usize {
        let addr = self.d(bus);
        self.read_u16(addr, bus).wrapping_add(self.y as u16) as usize
    }
    fn abs_off(&mut self, r: u8, bus: &mut impl HasAddressBus) -> usize {
        let addr = self.pc as usize;
        self.pc = self.pc.wrapping_add(2);
        let addr = self.read_u16(addr, bus).wrapping_add(r as u16);
        addr as usize
    }
    fn abs(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.abs_off(0, bus)
    }
    fn absx(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.abs_off(self.x, bus)
    }
    fn absy(&mut self, bus: &mut impl HasAddressBus) -> usize {
        self.abs_off(self.y, bus)
    }
    fn mb(&mut self, bus: &mut impl HasAddressBus) -> bool {
        let value = u16::from_le_bytes([
            bus.read(self.pc as usize),
            bus.read(self.pc.wrapping_add(1) as usize),
        ]);
        self.pc = self.pc.wrapping_add(2);
        let addr = value & 0x1FFF;
        let bit = value >> 13;
        let value = bus.read(addr as usize);
        (value & (0x01 << bit)) != 0
    }
    pub fn step(&mut self, bus: &mut impl HasAddressBus) {
        // Read opcode
        let opcode = bus.read(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        // Utility macro to create the YA register from the Y and A registers
        macro_rules! ya {
            () => {
                (self.y as u16 * 0x100 + self.a as u16)
            };
        }
        macro_rules! read_a_func {
            ($func: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                let val = bus.read(addr);
                self.a = self.$func(self.a, val);
            }};
        }
        /// Read a value into a register
        macro_rules! read_reg {
            ($register: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                self.$register = bus.read(addr);
                self.set_nz(self.$register);
            }};
        }
        /// Write a register's value
        macro_rules! write_reg {
            ($register: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                bus.write(addr, self.$register);
            }};
        }
        macro_rules! trans_reg {
            ($dst: ident, $src: ident) => {{
                self.$dst = self.$src;
                self.psw.z = self.$dst == 0;
                self.psw.n = (self.$dst & 0x80) != 0;
            }};
        }
        /// Reads 2 values using 2 different addressing mode(s),
        /// and then writes a value using the first addressing mode.
        /// `target` is the function address that will be read and then written to.
        /// `operand` (optional) is the function address that is just read.
        macro_rules! read_read_write_func {
            ($target: ident, $func: ident) => {{
                let addr = self.$target(bus);
                let value = bus.read(addr);
                let value = self.$func(value);
                bus.write(addr, value);
            }};
            ($func: ident, $target: ident, $operand: ident) => {{
                let addr = self.$operand(bus);
                let r = bus.read(addr);
                let addr = self.$target(bus);
                let l = bus.read(addr);
                let val = self.$func(l, r);
                bus.write(addr, val);
            }};
        }
        /// Read a value, then call a function with that value
        macro_rules! read_func {
            ($func: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                let val = bus.read(addr);
                self.$func(val);
            }};
        }
        macro_rules! read_write_func {
            ($func: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                let value = self.$func(bus.read(addr));
                bus.write(addr, value);
            }};
            ($func: ident, $read_addr: ident, $write_addr: ident) => {{
                let addr = self.$read_addr(bus);
                let value = self.$func(bus.read(addr));
                let addr = self.$write_addr(bus);
                bus.write(addr, value);
            }};
        }
        macro_rules! read_read_func {
            ($lhs_addr: ident, $rhs_addr: ident, $func: ident) => {{
                // In the assembly, the second value is provided first
                let addr = self.$rhs_addr(bus);
                let rhs = bus.read(addr as usize);
                let addr = self.$lhs_addr(bus);
                let lhs = bus.read(addr as usize);
                self.$func(lhs, rhs);
            }};
        }
        macro_rules! read_bit_func {
            ($flag: ident, $op: tt, $negate: expr) => {{
                // Todo maybe: Move MB to a macro param
                let bit = self.mb(bus);
                let val = (bit $op self.psw.$flag);
                self.psw.$flag = if $negate { !val } else { val };
            }};
            ($flag: ident, $op: tt) => {{ read_bit_func!($flag, $op, false) }};
        }
        macro_rules! branch_d_if_bit_eq {
            ($val: expr) => {{
                let bit = opcode >> 5;
                let addr = self.d(bus);
                let val = bus.read(addr as usize);
                self.read_and_branch_if((val >> bit) & 0x01 == $val, bus);
            }};
        }
        macro_rules! branch_on_flag {
            ($flag: ident, $val: expr) => {{
                self.read_and_branch_if(u8::from(self.psw.$flag) == $val, bus);
            }};
        }
        macro_rules! cbne {
            ($addr: ident) => {{
                let addr = self.$addr(bus);
                let value = bus.read(addr as usize);
                self.read_and_branch_if(self.a != value, bus);
            }};
        }
        macro_rules! set_flag {
            ($flag: ident, $val: expr) => {{
                self.psw.$flag = $val;
            }};
        }
        macro_rules! pop_reg {
            ($r: ident) => {{
                self.$r = self.pull_from_stack_u8(bus);
            }};
        }
        macro_rules! push_reg {
            ($r: ident) => {{
                self.push_to_stack_u8(self.$r, bus);
            }};
        }
        // When the bit location is in the top 3 bits of the opcode
        macro_rules! opcode_bit_func {
            ($addr: ident) => {{
                let bit = opcode >> 5;
                let addr = self.$addr(bus);
                let val = bus.read(addr as usize);
                (bit, addr, val)
            }};
        }
        // When the bit location is in the top 3 bits of the address
        macro_rules! addr_bit_func {
            ($addr: ident) => {{
                let addr = self.$addr(bus);
                let bit = addr >> 13;
                let addr = addr & 0x1FFF;
                let val = bus.read(addr as usize);
                (bit, addr, val)
            }};
        }
        match opcode {
            ADC_A_ABS => read_a_func!(adc, abs),
            ADC_A_ABSX => read_a_func!(adc, absx),
            ADC_A_ABSY => read_a_func!(adc, absy),
            ADC_A_D => read_a_func!(adc, d),
            ADC_A_DX => read_a_func!(adc, dx),
            ADC_A_IDX => read_a_func!(adc, idx),
            ADC_A_IDY => read_a_func!(adc, idy),
            ADC_A_IX => read_a_func!(adc, ix),
            ADC_A_IMM => read_a_func!(adc, imm),
            ADC_IX_IY => read_read_write_func!(adc, ix, iy),
            ADC_D_D => read_read_write_func!(adc, d, d),
            ADC_D_IMM => read_read_write_func!(adc, d, imm),
            ADDW_YA_D => {
                // Address low high
                let [al, ah] = self.dw(bus);
                let l = bus.read(al);
                let h = bus.read(ah);
                self.psw.c = false;
                self.a = self.adc(l, self.a);
                self.y = self.adc(h, self.y);
                self.set_nz_16le(self.a, self.y);
            }
            AND_IX_IY => read_read_write_func!(and, ix, iy),
            AND_A_IMM => read_a_func!(and, imm),
            AND_A_IX => read_a_func!(and, ix),
            AND_A_IDY => read_a_func!(and, idy),
            AND_A_IDX => read_a_func!(and, idx),
            AND_A_D => read_a_func!(and, d),
            AND_A_DX => read_a_func!(and, dx),
            AND_A_ABS => read_a_func!(and, abs),
            AND_A_ABSX => read_a_func!(and, absx),
            AND_A_ABSY => read_a_func!(and, absy),
            AND_D_D => read_read_write_func!(and, d, d),
            AND_D_IMM => read_read_write_func!(and, d, imm),
            AND1_C_NMB => read_bit_func!(c, &, true),
            AND1_C_MB => read_bit_func!(c, &),
            ASL_A => {
                self.a = self.asl(self.a);
            }
            ASL_D => read_read_write_func!(d, asl),
            ASL_DX => read_read_write_func!(dx, asl),
            ASL_ABS => read_read_write_func!(abs, asl),
            // BBS
            opcode if opcode & 0x1F == BBS_D_R_MASK => branch_d_if_bit_eq!(1),
            // BBC
            opcode if opcode & 0x1F == BBC_D_R_MASK => branch_d_if_bit_eq!(0),
            BCC_R => branch_on_flag!(c, 0),
            BCS_R => branch_on_flag!(c, 1),
            BEQ_R => branch_on_flag!(z, 1),
            BNE_R => branch_on_flag!(z, 0),
            BMI_R => branch_on_flag!(n, 1),
            BPL_R => branch_on_flag!(n, 0),
            BVC_R => branch_on_flag!(v, 0),
            BVS_R => branch_on_flag!(v, 1),
            BRA_R => self.branch_imm(bus),
            BRK => {
                self.push_to_stack_u16(self.pc, bus);
                self.push_to_stack_u8(self.psw.to_byte(), bus);
                self.psw.b = true;
                self.psw.i = false;
                self.pc = u16::from_le_bytes([bus.read(0xFFDE), bus.read(0xFFDF)]);
            }
            CALL_ABS => {
                let addr = self.abs(bus);
                self.call(addr as u16, bus);
            }
            CBNE_DX_R => cbne!(dx),
            CBNE_D_R => cbne!(d),
            opcode if opcode & 0x1F == CLR1_D => {
                let (bit, addr, val) = opcode_bit_func!(d);
                let val = val & !(0x01 << bit);
                bus.write(addr as usize, val);
            }
            CLRC => set_flag!(c, false),
            CLRP => set_flag!(p, false),
            CLRV => {
                set_flag!(v, false);
                set_flag!(h, false);
            }
            CMP_A_IMM => read_func!(cmp_a, imm),
            CMP_A_IX => read_func!(cmp_a, ix),
            CMP_A_IDY => read_func!(cmp_a, idy),
            CMP_A_IDX => read_func!(cmp_a, idx),
            CMP_A_D => read_func!(cmp_a, d),
            CMP_A_DX => read_func!(cmp_a, dx),
            CMP_A_ABS => read_func!(cmp_a, abs),
            CMP_A_ABSX => read_func!(cmp_a, absx),
            CMP_A_ABSY => read_func!(cmp_a, absy),
            CMP_IX_IY => read_read_func!(ix, iy, cmp),
            CMP_D_D => read_read_func!(d, d, cmp),
            CMP_D_IMM => read_read_func!(d, imm, cmp),
            CMP_X_IMM => read_func!(cmp_x, imm),
            CMP_X_D => read_func!(cmp_x, d),
            CMP_X_ABS => read_func!(cmp_x, abs),
            CMP_Y_IMM => read_func!(cmp_y, imm),
            CMP_Y_D => read_func!(cmp_y, d),
            CMP_Y_ABS => read_func!(cmp_y, abs),
            CMPW_YA_D => {
                let [al, ah] = self.dw(bus);
                let l = bus.read(al);
                let h = bus.read(ah);
                self.psw.c = true;
                self.cmp(self.a, l);
                self.cmp(self.y, h);
            }
            DAA_A => {
                if self.psw.c || self.a > 0x99 {
                    self.a = self.a.wrapping_add(0x60);
                    self.psw.c = true;
                }
                if self.psw.h || (self.a & 0x0F) > 0x09 {
                    self.a = self.a.wrapping_add(0x06);
                }
                self.set_nz(self.a);
            }
            DAS_A => {
                if !self.psw.c || self.a > 0x99 {
                    self.a = self.a.wrapping_sub(0x60);
                    self.psw.c = false;
                }
                if !self.psw.h || (self.a & 0x0F) > 0x09 {
                    self.a = self.a.wrapping_sub(0x06)
                }
                self.set_nz(self.a);
            }
            DBNZ_Y_R => {
                self.y = self.y.wrapping_sub(1);
                self.read_and_branch_if(self.y != 0, bus);
            }
            DBNZ_D_R => {
                let addr = self.d(bus);
                let val = bus.read(addr as usize).wrapping_sub(1);
                bus.write(addr as usize, val);
                self.read_and_branch_if(val != 0, bus);
            }
            DEC_A => self.a = self.dec(self.a),
            DEC_X => self.x = self.dec(self.x),
            DEC_Y => self.y = self.dec(self.y),
            DEC_D => read_write_func!(dec, d),
            DEC_DX => read_write_func!(dec, dx),
            DEC_ABS => read_write_func!(dec, abs),
            DECW_D => {
                let [al, ah] = self.dw(bus);
                let l = bus.read(al);
                let mut h = bus.read(ah);
                let (l, c) = l.overflowing_sub(1);
                bus.write(al, l);
                if c {
                    h = h.wrapping_sub(1);
                    bus.write(ah, h);
                    self.psw.z = false;
                }
                self.psw.z = l == 0 && h == 0;
                self.psw.n = (h & 0x80) != 0;
            }
            DI => self.psw.i = false,
            DIV_YA_X => {
                self.psw.h = (self.x & 0x0F) <= (self.y & 0x0F);
                (0..11).for_each(|_| bus.io());
                let ya = ya!() as u32;
                let x = self.x as u32;
                let (q, r) = if (self.y as u32) < (x << 1) {
                    (ya / x, ya % x)
                } else {
                    (
                        (255 - ((ya - (x << 9)) / (256 - x))),
                        (x + (ya - (x << 9)) % (256 - x)),
                    )
                };
                self.psw.v = if self.x == 0 {
                    true
                } else {
                    ya!() / self.x as u16 > 0xFF
                };
                self.a = (q & 0xFF) as u8;
                self.y = (r & 0xFF) as u8;
                self.set_nz(self.a);
            }
            EI => self.psw.i = true,
            EOR_IX_IY => read_read_write_func!(eor, ix, iy),
            EOR_A_IMM => read_a_func!(eor, imm),
            EOR_A_IX => read_a_func!(eor, ix),
            EOR_A_IDY => read_a_func!(eor, idy),
            EOR_A_IDX => read_a_func!(eor, idx),
            EOR_A_D => read_a_func!(eor, d),
            EOR_A_DX => read_a_func!(eor, dx),
            EOR_A_ABS => read_a_func!(eor, abs),
            EOR_A_ABSX => read_a_func!(eor, absx),
            EOR_A_ABSY => read_a_func!(eor, absy),
            EOR_D_D => read_read_write_func!(eor, d, d),
            EOR_D_IMM => read_read_write_func!(eor, d, imm),
            EOR1_C_MB => {
                let (bit, _addr, val) = addr_bit_func!(abs);
                self.psw.c ^= ((val >> bit) & 0x01) != 0;
            }
            INC_A => self.a = self.inc(self.a),
            INC_X => self.x = self.inc(self.x),
            INC_Y => self.y = self.inc(self.y),
            INC_D => read_write_func!(inc, d),
            INC_DX => read_write_func!(inc, dx),
            INC_ABS => read_write_func!(inc, abs),
            INCW_D => {
                let [al, ah] = self.dw(bus);
                let l = bus.read(al);
                let h = bus.read(ah);
                let (l, c) = l.overflowing_add(1);
                bus.write(al, l);
                let h = h.wrapping_add(u8::from(c));
                bus.write(ah, h);
                self.set_nz_16le(l, h);
            }
            JMP_IAX => {
                let addr = self.absx(bus);
                let addr = self.read_u16(addr, bus);
                self.pc = addr;
            }
            JMP_ABS => {
                self.pc = self.abs(bus) as u16;
            }
            LSR_A => self.a = self.lsr(self.a),
            LSR_D => read_write_func!(lsr, d),
            LSR_DX => read_write_func!(lsr, dx),
            LSR_ABS => read_write_func!(lsr, abs),
            MOV_IX_A => write_reg!(a, ix),
            MOV_IDY_A => write_reg!(a, idy),
            MOV_IDX_A => write_reg!(a, idx),
            MOV_DX_A => write_reg!(a, dx),
            MOV_DX_Y => write_reg!(y, dx),
            MOV_DY_X => write_reg!(x, dy),
            MOV_D_A => write_reg!(a, d),
            MOV_D_X => write_reg!(x, d),
            MOV_D_Y => write_reg!(y, d),
            MOV_ABSX_A => write_reg!(a, absx),
            MOV_ABSY_A => write_reg!(a, absy),
            MOV_ABS_A => write_reg!(a, abs),
            MOV_ABS_X => write_reg!(x, abs),
            MOV_ABS_Y => write_reg!(y, abs),
            MOV_A_IMM => read_reg!(a, imm),
            MOV_A_IX => read_reg!(a, ix),
            MOV_A_IDY => read_reg!(a, idy),
            MOV_A_IDX => read_reg!(a, idx),
            MOV_A_D => read_reg!(a, d),
            MOV_A_DX => read_reg!(a, dx),
            MOV_A_ABS => read_reg!(a, abs),
            MOV_A_ABSX => read_reg!(a, absx),
            MOV_A_ABSY => read_reg!(a, absy),
            MOV_X_IMM => read_reg!(x, imm),
            MOV_X_D => read_reg!(x, d),
            MOV_X_DY => read_reg!(x, dy),
            MOV_X_ABS => read_reg!(x, abs),
            MOV_Y_IMM => read_reg!(y, imm),
            MOV_Y_D => read_reg!(y, d),
            MOV_Y_DX => read_reg!(y, dx),
            MOV_Y_ABS => read_reg!(y, abs),
            MOV_A_X => trans_reg!(a, x),
            MOV_A_Y => trans_reg!(a, y),
            MOV_SP_X => {
                // No flags set
                self.sp = self.x;
            }
            MOV_X_A => trans_reg!(x, a),
            MOV_X_SP => trans_reg!(x, sp),
            MOV_A_XINC => {
                read_reg!(a, ix);
                self.x = self.x.wrapping_add(1);
            }
            MOV_XINC_A => {
                write_reg!(a, ix);
                self.x = self.x.wrapping_add(1);
            }
            MOV_Y_A => trans_reg!(y, a),
            MOV_D_D => read_write_func!(mov_no_flags, d, d),
            MOV_D_IMM => read_write_func!(mov_no_flags, imm, d),
            MOV1_C_MB => {
                let (bit, _addr, val) = addr_bit_func!(abs);
                self.psw.c = (val & (0x01 << bit)) != 0;
            }
            MOV1_MB_C => {
                let (bit, addr, val) = addr_bit_func!(abs);
                let mask = 1 << bit;
                let val = if self.psw.c { val | mask } else { val & !mask };
                bus.write(addr as usize, val);
            }
            MOVW_D_YA => {
                // let addr = self.d(bus);
                // self.write_u16(addr, ya!(), bus);
                let [al, ah] = self.dw(bus);
                bus.write(al, self.a);
                bus.write(ah, self.y);
            }
            MOVW_YA_D => {
                let [al, ah] = self.dw(bus);
                let low = bus.read(al);
                let high = bus.read(ah);
                self.a = low;
                self.y = high;
                self.set_nz_16le(self.a, self.y);
                // let addr = self.d(bus);
                // let [low, high] = self.read_u16(addr, bus).to_le_bytes();
                // self.a = low;
                // self.y = high;
            }
            MUL_YA => {
                let res = self.y as u16 * self.a as u16;
                self.a = (res & 0xFF) as u8;
                self.y = (res >> 8) as u8;
                // Only uses Y for flags (i guess)
                self.set_nz(self.y);
            }
            NOP => {}
            NOT1_MB => {
                let (bit, addr, val) = addr_bit_func!(abs);
                let val = val ^ (0x01 << bit);
                bus.write(addr as usize, val);
            }
            NOTC => self.psw.c = !self.psw.c,
            OR_IX_IY => read_read_write_func!(or, ix, iy),
            OR_A_IMM => read_func!(or_a, imm),
            OR_A_IX => read_func!(or_a, ix),
            OR_A_IDY => read_func!(or_a, idy),
            OR_A_IDX => read_func!(or_a, idx),
            OR_A_D => read_func!(or_a, d),
            OR_A_DX => read_func!(or_a, dx),
            OR_A_ABS => read_func!(or_a, abs),
            OR_A_ABSX => read_func!(or_a, absx),
            OR_A_ABSY => read_func!(or_a, absy),
            OR_D_D => read_read_write_func!(or, d, d),
            OR_D_IMM => read_read_write_func!(or, d, imm),
            OR1_C_MB => {
                let (bit, _addr, val) = addr_bit_func!(abs);
                self.psw.c |= (val & (0x01 << bit)) != 0;
            }
            OR1_C_NMB => {
                let (bit, _addr, val) = addr_bit_func!(abs);
                self.psw.c |= !((val & (0x01 << bit)) != 0);
            }
            PCALL => {
                let addr = self.imm(bus);
                self.call(0xFF00 + bus.read(addr as usize) as u16, bus);
            }
            POP_A => pop_reg!(a),
            POP_X => pop_reg!(x),
            POP_Y => pop_reg!(y),
            PUSH_A => push_reg!(a),
            PUSH_X => push_reg!(x),
            PUSH_Y => push_reg!(y),
            PUSH_PSW => self.push_to_stack_u8(self.psw.to_byte(), bus),
            POP_PSW => self.psw = ProgramStatusWord::from_byte(self.pull_from_stack_u8(bus)),
            RET => self.pc = self.pull_from_stack_u16(bus),
            RETI => {
                self.psw = ProgramStatusWord::from_byte(self.pull_from_stack_u8(bus));
                self.pc = self.pull_from_stack_u16(bus);
            }
            ROL_A => self.a = self.rol(self.a),
            ROL_D => read_write_func!(rol, d),
            ROL_DX => read_write_func!(rol, dx),
            ROL_ABS => read_write_func!(rol, abs),
            ROR_A => self.a = self.ror(self.a),
            ROR_D => read_write_func!(ror, d),
            ROR_DX => read_write_func!(ror, dx),
            ROR_ABS => read_write_func!(ror, abs),
            SBC_IX_IY => read_read_write_func!(sbc, ix, iy),
            SBC_A_IMM => read_a_func!(sbc, imm),
            SBC_A_IX => read_a_func!(sbc, ix),
            SBC_A_IDY => read_a_func!(sbc, idy),
            SBC_A_IDX => read_a_func!(sbc, idx),
            SBC_A_D => read_a_func!(sbc, d),
            SBC_A_DX => read_a_func!(sbc, dx),
            SBC_A_ABS => read_a_func!(sbc, abs),
            SBC_A_ABSX => read_a_func!(sbc, absx),
            SBC_A_ABSY => read_a_func!(sbc, absy),
            SBC_D_D => read_read_write_func!(sbc, d, d),
            SBC_D_IMM => read_read_write_func!(sbc, d, imm),
            SUBW_YA_D => {
                // Address low high
                let [al, ah] = self.dw(bus);
                let l = bus.read(al);
                let h = bus.read(ah);
                self.psw.c = true;
                self.a = self.sbc(self.a, l);
                self.y = self.sbc(self.y, h);
                self.set_nz_16le(self.a, self.y);
            }
            opcode if opcode & 0x1F == SET1_MASK => {
                let (bit, addr, val) = opcode_bit_func!(d);
                let val = val | (0x01 << bit);
                bus.write(addr as usize, val);
            }
            SETC => self.psw.c = true,
            SETP => self.psw.p = true,
            opcode if opcode & 0x0F == TCALL_MASK => {
                let offset = opcode >> 4;
                let addr = self.read_u16(0xFFDE - 2 * offset as usize, bus);
                self.call(addr, bus);
            }
            TCLR1_ABS => read_write_func!(tclr, abs),
            TSET1_ABS => read_write_func!(tset, abs),
            XCN_A => {
                self.a = self.a.rotate_left(4);
                self.set_nz(self.a);
            }
            _ => panic!("Unimplemented SPC700 opcode: {:2X}", opcode),
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Processor {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0xFFC0,
            psw: ProgramStatusWord::default(),
        }
    }
}
