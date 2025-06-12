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

    /// Add two bytes together and calculate flags
    /// Returns in the format (value, carry, zero, negative, (signed) overflow)
    fn add_bytes(a: u8, b: u8, carry: bool) -> (u8, bool, bool, bool, bool) {
        let result = a.wrapping_add(b).wrapping_add(carry.into());
        return (
            result,
            // Carry flag
            result < a,
            // Zero flag
            result == 0,
            // Negative flag
            result > 0x7F,
            // Overflow flag
            ((result ^ a as u8) & (result ^ b)) & 0x80 == 0,
        );
    }

    /// Add with Carry
    fn adc(&mut self, addr: u24, memory: &mut impl Memory) {
        let (value, c, z, n, v) = Processor::add_bytes(self.a, memory.read(addr.into()), self.p.c);
        self.a = value;
        if self.p.is_16bit() {
            let (value, c, z2, n, v) =
                Processor::add_bytes(self.b, memory.read(addr.wrapping_add(1u32).into()), c);
            self.b = value;
            // Both need to be 0 for the zero flag to be set
            self.p.z = z && z2;
            self.p.n = n;
            self.p.v = v;
            self.p.c = c;
        } else {
            self.p.z = z;
            self.p.n = n;
            self.p.v = v;
            self.p.c = c;
        }
    }
    /// And with accumulator
    fn and(&mut self, addr: u24, memory: &impl Memory) {
        self.a = self.a & memory.read(addr.into());
        if self.p.is_16bit() {
            self.b = self.b & memory.read(addr.wrapping_add(1u32).into());
            self.p.z = self.a == 0 && self.b == 0;
            self.p.n = self.b > 0x7F;
        } else {
            self.p.n = self.a > 0x7F;
            self.p.z = self.a == 0;
        }
    }
    /// Shift a u8 and set the appropriate flags
    fn asl_8(&mut self, value: u8) -> u8 {
        let (value, carry) = value.overflowing_shl(1);
        self.p.c = carry;
        self.p.n = value > 0x7F;
        self.p.z = value == 0;
        value
    }
    fn asl_16(&mut self, low: u8, high: u8) -> (u8, u8) {
        let low = self.asl_8(low);
        let carry = self.p.c;
        let zero = self.p.z;
        let high = self.asl_8(high).wrapping_add(carry.into());
        // Both need to be 0
        self.p.z = self.p.z & zero;
        (low, high)
    }
    /// Arithmatic shift left
    fn asl(&mut self, addr: u24, memory: &mut impl Memory) {
        if self.p.is_8bit() {
            let value = memory.read(addr.into());
            memory.write(addr.into(), value);
        } else {
            let (low, high) = self.asl_16(
                memory.read(addr.into()),
                memory.read(addr.wrapping_add(1u32).into()),
            );
            memory.write(addr.into(), low);
            memory.write(addr.wrapping_add(1u32).into(), high);
        }
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
        self.a_off(memory, self.x)
    }
    /// Absolute Y Indexed addressing
    fn ay(&mut self, memory: &mut impl Memory) -> u24 {
        self.a_off(memory, self.y)
    }
    /// Absolute Long addressing
    fn al(&mut self, memory: &mut impl Memory) -> u24 {
        let addr = u24::from(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(3);
        read_u24(memory, addr)
    }
    /// Absolute Long X Indexed
    fn alx(&mut self, memory: &mut impl Memory) -> u24 {
        self.al(memory).wrapping_add(self.x)
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
        self.d_off(memory, self.x)
    }
    /// Direct Y Indexed addressing
    fn dy(&mut self, memory: &mut impl Memory) -> u24 {
        self.d_off(memory, self.y)
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
        addr.wrapping_add(self.x).with_bank(self.dbr)
    }
    /// Direct Indirect Y Indexed addressing
    fn diy(&mut self, memory: &mut impl Memory) -> u24 {
        let addr: u24 = self.di(memory).into();
        memory.io();
        addr.wrapping_add(self.y)
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
        addr.wrapping_add(self.y)
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
        addr.wrapping_add(self.y)
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
        let opcode = read_u8(memory, u24::from(self.pbr, self.pc));
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
            AND_I => cpu_func!(and, i),
            AND_A => cpu_func!(and, a),
            AND_AL => cpu_func!(and, al),
            AND_D => cpu_func!(and, d),
            AND_DI => cpu_func!(and, di),
            AND_DIL => cpu_func!(and, dil),
            AND_AX => cpu_func!(and, ax),
            AND_ALX => cpu_func!(and, alx),
            AND_AY => cpu_func!(and, ay),
            AND_DX => cpu_func!(and, dx),
            AND_DIX => cpu_func!(and, dix),
            AND_DIY => cpu_func!(and, diy),
            AND_DILY => cpu_func!(and, dily),
            AND_SR => cpu_func!(and, sr),
            AND_SRIY => cpu_func!(and, sriy),
            ASL_ACC => {
                self.a = self.asl_8(self.a);
                if self.p.is_16bit() {
                    let carry = self.p.c;
                    self.b = self.asl_8(self.b).wrapping_add(carry.into());
                }
            }
            ASL_A => cpu_func!(asl, a),
            ASL_D => cpu_func!(asl, d),
            ASL_AX => cpu_func!(asl, ax),
            ASL_DX => cpu_func!(asl, dx),
            BCC => self.bcc(),
            BCS => self.bcs(),
            BEQ => self.beq(),
            BIT_I => cpu_func!(bit, i),
            BIT_A => cpu_func!(bit, a),
            BIT_D => cpu_func!(bit, d),
            BIT_AX => cpu_func!(bit, ax),
            BIT_DX => cpu_func!(bit, dx),
            BMI => self.bmi(),
            BNE => self.bne(),
            BPL => self.bpl(),
            BRA => self.bra(),
            BRK => self.brk(),
            BRL => self.brl(),
            BVC => self.bvc(),
            BVS => self.bvs(),
            CLC => self.clc(),
            CLD => self.cld(),
            CLI => self.cli(),
            CLV => self.clv(),
            CMP_I => cpu_func!(cmp, i),
            CMP_A => cpu_func!(cmp, a),
            CMP_AL => cpu_func!(cmp, al),
            CMP_D => cpu_func!(cmp, d),
            CMP_DI => cpu_func!(cmp, di),
            CMP_DIL => cpu_func!(cmp, dil),
            CMP_AX => cpu_func!(cmp, ax),
            CMP_ALX => cpu_func!(cmp, alx),
            CMP_AY => cpu_func!(cmp, ay),
            CMP_DX => cpu_func!(cmp, dx),
            CMP_DIX => cpu_func!(cmp, dix),
            CMP_DIY => cpu_func!(cmp, diy),
            CMP_DILY => cpu_func!(cmp, dily),
            CMP_SR => cpu_func!(cmp, sr),
            CMP_SRIY => cpu_func!(cmp, sriy),
            COP => self.cop(),
            CPX_I => cpu_func!(cpx, i),
            CPX_A => cpu_func!(cpx, a),
            CPX_D => cpu_func!(cpx, d),
            CPY_I => cpu_func!(cpy, i),
            CPY_A => cpu_func!(cpy, a),
            CPY_D => cpu_func!(cpy, d),
            DEC_ACC => cpu_func!(dec, acc),
            DEC_A => cpu_func!(dec, a),
            DEC_D => cpu_func!(dec, d),
            DEC_AX => cpu_func!(dec, ax),
            DEC_DX => cpu_func!(dec, dx),
            DEX => self.dex(),
            DEY => self.dey(),
            EOR_I => cpu_func!(eor, i),
            EOR_A => cpu_func!(eor, a),
            EOR_AL => cpu_func!(eor, al),
            EOR_D => cpu_func!(eor, d),
            EOR_DI => cpu_func!(eor, di),
            EOR_DIL => cpu_func!(eor, dil),
            EOR_AX => cpu_func!(eor, ax),
            EOR_ALX => cpu_func!(eor, alx),
            EOR_AY => cpu_func!(eor, ay),
            EOR_DX => cpu_func!(eor, dx),
            EOR_DIX => cpu_func!(eor, dix),
            EOR_DIY => cpu_func!(eor, diy),
            EOR_DILY => cpu_func!(eor, dily),
            EOR_SR => cpu_func!(eor, sr),
            EOR_SRIY => cpu_func!(eor, sriy),
            INC_ACC => cpu_func!(inc, acc),
            INC_A => cpu_func!(inc, a),
            INC_D => cpu_func!(inc, d),
            INC_AX => cpu_func!(inc, ax),
            INC_DX => cpu_func!(inc, dx),
            INX => self.inx(),
            INY => self.iny(),
            JMP_A => cpu_func!(jmp, a),
            JMP_AI => cpu_func!(jmp, ai),
            JMP_AIX => cpu_func!(jmp, aix),
            JMP_AL => cpu_func!(jmp, al),
            JMP_AIL => cpu_func!(jmp, ail),
            JSR_A => cpu_func!(jsr, a),
            JSR_AIX => cpu_func!(jsr, aix),
            JSR_AL => cpu_func!(jsr, al),
            LDA_I => cpu_func!(lda, i),
            LDA_A => cpu_func!(lda, a),
            LDA_AL => cpu_func!(lda, al),
            LDA_D => cpu_func!(lda, d),
            LDA_DI => cpu_func!(lda, di),
            LDA_DIL => cpu_func!(lda, dil),
            LDA_AX => cpu_func!(lda, ax),
            LDA_ALX => cpu_func!(lda, alx),
            LDA_AY => cpu_func!(lda, ay),
            LDA_DX => cpu_func!(lda, dx),
            LDA_DIX => cpu_func!(lda, dix),
            LDA_DIY => cpu_func!(lda, diy),
            LDA_DILY => cpu_func!(lda, dily),
            LDA_SR => cpu_func!(lda, sr),
            LDA_SRIY => cpu_func!(lda, sriy),
            LDX_I => cpu_func!(ldx, i),
            LDX_A => cpu_func!(ldx, a),
            LDX_D => cpu_func!(ldx, d),
            LDX_AY => cpu_func!(ldx, ay),
            LDX_DY => cpu_func!(ldx, dy),
            LDY_I => cpu_func!(ldy, i),
            LDY_A => cpu_func!(ldy, a),
            LDY_D => cpu_func!(ldy, d),
            LDY_AX => cpu_func!(ldy, ax),
            LDY_DX => cpu_func!(ldy, dx),
            LSR_ACC => cpu_func!(lsr, acc),
            LSR_A => cpu_func!(lsr, a),
            LSR_D => cpu_func!(lsr, d),
            LSR_AX => cpu_func!(lsr, ax),
            LSR_DX => cpu_func!(lsr, dx),
            MVN_NEXT => cpu_func!(mvn, next),
            MVN_PREV => cpu_func!(mvn, prev),
            NOP => self.nop(),
            ORA_I => cpu_func!(ora, i),
            ORA_A => cpu_func!(ora, a),
            ORA_AL => cpu_func!(ora, al),
            ORA_D => cpu_func!(ora, d),
            ORA_DI => cpu_func!(ora, di),
            ORA_DIL => cpu_func!(ora, dil),
            ORA_AX => cpu_func!(ora, ax),
            ORA_ALX => cpu_func!(ora, alx),
            ORA_AY => cpu_func!(ora, ay),
            ORA_DX => cpu_func!(ora, dx),
            ORA_DIX => cpu_func!(ora, dix),
            ORA_DIY => cpu_func!(ora, diy),
            ORA_DILY => cpu_func!(ora, dily),
            ORA_SR => cpu_func!(ora, sr),
            ORA_SRIY => cpu_func!(ora, sriy),
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
            REP_I => cpu_func!(rep, i),
            ROL_ACC => cpu_func!(rol, acc),
            ROL_A => cpu_func!(rol, a),
            ROL_D => cpu_func!(rol, d),
            ROL_AX => cpu_func!(rol, ax),
            ROL_DX => cpu_func!(rol, dx),
            ROR_ACC => cpu_func!(ror, acc),
            ROR_A => cpu_func!(ror, a),
            ROR_D => cpu_func!(ror, d),
            ROR_AX => cpu_func!(ror, ax),
            ROR_DX => cpu_func!(ror, dx),
            RTI => self.rti(),
            RTL => self.rtl(),
            RTS => self.rts(),
            SBC_I => cpu_func!(sbc, i),
            SBC_A => cpu_func!(sbc, a),
            SBC_AL => cpu_func!(sbc, al),
            SBC_D => cpu_func!(sbc, d),
            SBC_DI => cpu_func!(sbc, di),
            SBC_DIL => cpu_func!(sbc, dil),
            SBC_AX => cpu_func!(sbc, ax),
            SBC_ALX => cpu_func!(sbc, alx),
            SBC_AY => cpu_func!(sbc, ay),
            SBC_DX => cpu_func!(sbc, dx),
            SBC_DIX => cpu_func!(sbc, dix),
            SBC_DIY => cpu_func!(sbc, diy),
            SBC_DILY => cpu_func!(sbc, dily),
            SBC_SR => cpu_func!(sbc, sr),
            SBC_SRIY => cpu_func!(sbc, sriy),
            SEC => self.sec(),
            SED => self.sed(),
            SEI => self.sei(),
            SEP_I => cpu_func!(sep, i),
            STA_A => cpu_func!(sta, a),
            STA_AL => cpu_func!(sta, al),
            STA_D => cpu_func!(sta, d),
            STA_DI => cpu_func!(sta, di),
            STA_DIL => cpu_func!(sta, dil),
            STA_AX => cpu_func!(sta, ax),
            STA_ALX => cpu_func!(sta, alx),
            STA_AY => cpu_func!(sta, ay),
            STA_DX => cpu_func!(sta, dx),
            STA_DIX => cpu_func!(sta, dix),
            STA_DIY => cpu_func!(sta, diy),
            STA_DILY => cpu_func!(sta, dily),
            STA_SR => cpu_func!(sta, sr),
            STA_SRIY => cpu_func!(sta, sriy),
            STP => self.stp(),
            STX_A => cpu_func!(stx, a),
            STX_D => cpu_func!(stx, d),
            STX_DY => cpu_func!(stx, dy),
            STY_A => cpu_func!(sty, a),
            STY_D => cpu_func!(sty, d),
            STY_DX => cpu_func!(sty, dx),
            STZ_A => cpu_func!(stz, a),
            STZ_D => cpu_func!(stz, d),
            STZ_AX => cpu_func!(stz, ax),
            STZ_DX => cpu_func!(stz, dx),
            TAX => self.tax(),
            TAY => self.tay(),
            TCD => self.tcd(),
            TCS => self.tcs(),
            TDC => self.tdc(),
            TRB_A => cpu_func!(trb, a),
            TRB_D => cpu_func!(trb, d),
            TSB_A => cpu_func!(tsb, a),
            TSB_D => cpu_func!(tsb, d),
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
            XCE => self.xce(),
            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }
}
