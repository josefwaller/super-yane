use crate::{StatusRegister, opcodes::*};

#[derive(Copy, Clone)]
pub struct Processor {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub sr: StatusRegister,
}

pub trait HasAddressBus {
    fn io(&mut self);
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
}

impl Processor {
    fn push_to_stack_u8(&mut self, val: u8, bus: &mut impl HasAddressBus) {
        bus.write(self.sp as usize + 0x100, val);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pull_from_stack_u8(&mut self, bus: &mut impl HasAddressBus) -> u8 {
        let val = bus.read(self.sp as usize + 0x100);
        self.sp = self.sp.wrapping_add(1);
        val
    }
    fn push_to_stack_u16(&mut self, val: u16, bus: &mut impl HasAddressBus) {
        let [l, h] = val.to_le_bytes();
        self.push_to_stack_u8(h, bus);
        self.push_to_stack_u8(l, bus);
    }
    fn pull_from_stack_u16(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        let [h, l] = [self.pull_from_stack_u8(bus), self.pull_from_stack_u8(bus)];
        u16::from_le_bytes([l, h])
    }
    fn adc(&mut self, l: u8, r: u8) -> u8 {
        let (r, c1) = l.overflowing_add(r);
        let (r, c2) = r.overflowing_add(self.sr.c.into());
        self.sr.c = c1 | c2;
        // Todo: More flags
        r
    }
    fn addw(&mut self, l: u16, r: u16) -> u16 {
        let (r, c1) = l.overflowing_add(r);
        let (r, c2) = r.overflowing_add(self.sr.c.into());
        self.sr.c = c1 | c2;
        r
    }
    fn and(&mut self, l: u8, r: u8) -> u8 {
        let v = l & r;
        self.sr.z = v == 0;
        self.sr.n = (v & 0x80) != 0;
        v
    }
    fn asl(&mut self, v: u8) -> u8 {
        let v = v.rotate_left(1);
        self.sr.c = (v & 0x01) != 0;
        v & 0xFE
    }
    fn branch_imm(&mut self, bus: &mut impl HasAddressBus) {
        let offset_addr = self.imm(bus);
        let offset = bus.read(offset_addr as usize);
        self.pc =
            ((self.pc as isize).wrapping_add((offset as i8) as isize) % u16::MAX as isize) as u16;
    }
    fn cmp(&mut self, a: u8, b: u8) {
        let (r, c) = a.overflowing_sub(b);
        self.sr.z = r == 0;
        self.sr.n = (r & 0x80) != 0;
        self.sr.c = c;
    }
    fn cmp_a(&mut self, v: u8) {
        self.cmp(v, self.a);
    }
    fn cmp_x(&mut self, v: u8) {
        self.cmp(v, self.x);
    }
    fn cmp_y(&mut self, v: u8) {
        self.cmp(v, self.y)
    }
    fn cmp_w(&mut self, a: u16, b: u16) {
        let (r, c) = a.overflowing_sub(b);
        self.sr.z = r == 0;
        self.sr.n = (r & 0x8000) != 0;
        self.sr.c = c;
    }

    /// Immediate addressing
    fn imm(&mut self, _bus: &mut impl HasAddressBus) -> u16 {
        self.pc as u16
    }
    /// Direct page addressing
    fn d(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        let addr = bus.read(self.pc as usize) as u16;
        self.pc = self.pc.wrapping_add(1);
        0x100 * u16::from(self.sr.p) + addr
    }
    /// Direct page with X offset
    fn dx(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.d(bus).wrapping_add(self.x as u16)
    }
    /// Direct page with Y offset
    fn dy(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.d(bus).wrapping_add(self.y as u16)
    }
    fn id(&mut self, r: u8, bus: &mut impl HasAddressBus) -> u16 {
        // Todo: Page wrap
        let addr = self.d(bus).wrapping_add(r as u16);
        // Read the pointer
        bus.read(addr as usize) as u16 + 0x100 * bus.read(addr.wrapping_add(1) as usize) as u16
    }
    fn ix(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        bus.read(self.x as usize) as u16
    }
    fn iy(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        bus.read(self.y as usize) as u16
    }
    fn idx(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.id(self.x, bus)
    }
    fn idy(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.id(self.y, bus)
    }
    fn abs(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        let addr = bus.read(self.pc as usize) as u16
            + 0x100 * bus.read(self.pc.wrapping_add(1) as usize) as u16;
        self.pc = self.pc.wrapping_add(2);
        addr
    }
    fn absx(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.abs(bus).wrapping_add(self.x as u16)
    }
    fn absy(&mut self, bus: &mut impl HasAddressBus) -> u16 {
        self.abs(bus).wrapping_add(self.y as u16)
    }
    fn mb(&mut self, bus: &mut impl HasAddressBus) -> bool {
        let value = u16::from_le_bytes([
            bus.read(self.pc as usize),
            bus.read(self.pc.wrapping_add(1) as usize),
        ]);
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
            ($read: ident, $func: ident) => {{
                let addr = self.$read(bus);
                let val = bus.read(addr as usize);
                self.a = self.$func(val, self.a);
            }};
        }
        /// Reads 2 values using 2 different addressing mode(s),
        /// and then writes a value using the first addressing mode.
        /// `target` is the function address that will be read and then written to.
        /// `operand` (optional) is the function address that is just read.
        macro_rules! read_read_write_func {
            ($target: ident, $func: ident) => {{
                let addr = self.$target(bus) as usize;
                let value = bus.read(addr);
                let value = self.$func(value);
                bus.write(addr, value);
            }};
            ($target: ident, $operand: ident, $func: ident) => {{
                let addr = self.$operand(bus) as usize;
                let l = bus.read(addr);
                let addr = self.$target(bus) as usize;
                let r = bus.read(addr);
                let val = self.$func(l, r);
                bus.write(addr, val);
            }};
        }
        /// Read a value, then call a function with that value
        macro_rules! read_func {
            ($func: ident, $addr: ident) => {{
                let addr = self.$addr(bus);
                let val = bus.read(addr as usize);
                self.$func(val);
            }};
        }
        /// Reads a value, operates on it with the YA register, and
        /// then stores the result in the YA register
        macro_rules! read_ya_func {
            ($operand: ident, $func: ident) => {{
                let addr = self.$operand(bus) as usize;
                let value = u16::from_le_bytes([bus.read(addr), bus.read(addr + 1)]);
                let result = self.$func(self.y as u16 * 0x100 + self.a as u16, value);
                self.y = (result >> 8) as u8;
                self.a = (result & 0xFF) as u8;
            }};
        }
        macro_rules! read_read_func {
            ($addr_one: ident, $addr_two: ident, $func: ident) => {{
                let addr = self.$addr_one(bus);
                let val_one = bus.read(addr as usize);
                let addr = self.$addr_two(bus);
                let val_two = bus.read(addr as usize);
                self.$func(val_two, val_one);
            }};
        }
        macro_rules! read_bit_func {
            ($flag: ident, $op: tt, $negate: expr) => {{
                // Todo maybe: Move MB to a macro param
                let bit = self.mb(bus);
                let val = (bit $op self.sr.$flag);
                self.sr.$flag = if $negate { !val } else { val };
            }};
            ($flag: ident, $op: tt) => {{ read_bit_func!($flag, $op, false) }};
        }
        macro_rules! branch_d_if_bit_eq {
            ($val: expr) => {{
                let bit = opcode >> 5;
                let addr = self.d(bus);
                let val = bus.read(addr as usize);
                if (val >> bit) & 0x01 == $val {
                    self.branch_imm(bus);
                }
            }};
        }
        macro_rules! branch_on_flag {
            ($flag: ident, $val: expr) => {{
                if u8::from(self.sr.$flag) == $val {
                    self.branch_imm(bus);
                }
            }};
        }
        macro_rules! cbne {
            ($addr: ident) => {{
                let addr = self.$addr(bus);
                let value = bus.read(addr as usize);
                if self.a == value {
                    self.branch_imm(bus);
                }
            }};
        }
        macro_rules! set_flag {
            ($flag: ident, $val: expr) => {{
                self.sr.$flag = $val;
            }};
        }
        match opcode {
            ADC_A_ABS => read_a_func!(abs, adc),
            ADC_A_ABSX => read_a_func!(absx, adc),
            ADC_A_ABSY => read_a_func!(absy, adc),
            ADC_A_D => read_a_func!(d, adc),
            ADC_A_DX => read_a_func!(dx, adc),
            ADC_A_IDX => read_a_func!(idx, adc),
            ADC_A_IDY => read_a_func!(idy, adc),
            ADC_A_IX => read_a_func!(ix, adc),
            ADC_A_IMM => read_a_func!(imm, adc),
            ADC_IX_IY => read_read_write_func!(ix, iy, adc),
            ADC_D_D => read_read_write_func!(d, d, adc),
            ADC_D_IMM => read_read_write_func!(d, imm, adc),
            ADDW_YA_D => read_ya_func!(d, addw),
            AND_IX_IY => read_read_write_func!(ix, iy, and),
            AND_A_IMM => read_a_func!(imm, and),
            AND_A_IX => read_a_func!(ix, and),
            AND_A_IDY => read_a_func!(idy, and),
            AND_A_IDX => read_a_func!(idx, and),
            AND_A_D => read_a_func!(d, and),
            AND_A_DX => read_a_func!(dx, and),
            AND_A_ABS => read_a_func!(abs, and),
            AND_A_ABSX => read_a_func!(absx, and),
            AND_A_ABSY => read_a_func!(absy, and),
            AND_D_D => read_read_write_func!(d, d, and),
            AND_D_IMM => read_read_write_func!(d, imm, and),
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
                self.push_to_stack_u8(self.sr.to_byte(), bus);
                self.pc = bus.read(0xFFDE) as u16 + 0x100 * bus.read(0xFFDF) as u16;
            }
            CALL_ABS => {
                self.push_to_stack_u16(self.pc, bus);
                self.pc = self.abs(bus);
            }
            CBNE_DX_R => cbne!(dx),
            CBNE_D_R => cbne!(d),
            opcode if opcode & 0x1F == CLR1_D => {
                let bit = opcode >> 5;
                let addr = self.d(bus);
                let val = bus.read(addr as usize) & !(0x01 << bit);
                bus.write(addr as usize, val);
            }
            CLRC => set_flag!(c, false),
            CLRP => set_flag!(p, false),
            CLRV => set_flag!(v, false),
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
                let addr = self.d(bus);
                let val = bus.read(addr as usize) as u16
                    + 0x100 * bus.read(addr.wrapping_add(1) as usize) as u16;
                self.cmp_w(ya!(), val);
            }
            _ => {}
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
            pc: 0,
            sr: StatusRegister::default(),
        }
    }
}
