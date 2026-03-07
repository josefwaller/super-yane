use derivative::Derivative;
use paste::paste;
use wdc65816::{HasAddressBus, Processor, opcodes::*};

#[derive(Derivative)]
#[derivative(Default)]
struct SimpleMemory {
    #[derivative(Default(value = "[0; 0x10_0000]"))]
    memory: [u8; 0x10_0000],
    num_cycles: usize,
}

impl HasAddressBus for SimpleMemory {
    fn read(&mut self, address: usize) -> u8 {
        self.num_cycles += 1;
        self.memory[address]
    }
    fn write(&mut self, address: usize, value: u8) {
        self.num_cycles += 1;
        self.memory[address] = value
    }
    fn io(&mut self) {
        self.num_cycles += 1;
    }
}

/// Returns (read, write, io)
fn get_timings_for_opcode(opcode: u8, p: &Processor) -> usize {
    // Offsets
    // A/X/Y is 16bit
    let a = if p.p.a_is_16bit() || p.p.xy_is_16bit() {
        1
    } else {
        0
    };
    // DL is not 0
    let d = if p.dl != 0 { 1 } else { 0 };
    // Emulation mode
    let e = if p.p.e { 1 } else { 0 };
    // Page cross
    let pc = 0;
    match opcode {
        // 1a. Absolute a
        ADC_A | AND_A | BIT_A | CMP_A | CPX_A | CPY_A | EOR_A | LDA_A | LDX_A | LDY_A | ORA_A
        | SBC_A | STA_A | STX_A | STY_A | STZ_A => 4 + a,
        // 1b. Absolute a
        JMP_A => 3,
        // 1c. Absolute a
        JSR_A => 6,
        // 1d. Absolute (R-M-W) a
        ASL_A | DEC_A | INC_A | LSR_A | ROL_A | ROR_A | TRB_A | TSB_A => 6 + 2 * a,
        // 2a. Absolute Indexed Indirect (a,x)
        JMP_AIX => 6,
        // 2b. Absolute Indexed Indirect (a,x)
        JSR_AIX => 8,
        // 3a. Absolute Indirect (a)
        JMP_AIL => 6,
        // 3b. Absolute Indirect (a)
        JMP_AI => 5,
        // 4a. Absolute Long al
        ADC_AL | AND_AL | CMP_AL | EOR_AL | LDA_AL | ORA_AL | SBC_AL | STA_AL => 5 + a,
        // 4b. Absolute Long (JUMP) al
        JMP_AL => 4,
        // 4c. Absolute Long (JUMP to Subroutine Long) al
        JSL => 8,
        // 5. Absolute Long,X al,x
        ADC_ALX | AND_ALX | CMP_ALX | EOR_ALX | LDA_ALX | ORA_ALX | SBC_ALX | STA_ALX => 5 + a,
        // 6a Absolute, X a, x
        ADC_AX | AND_AX | BIT_AX | CMP_AX | EOR_AX | LDA_AX | LDY_AX | ORA_AX | SBC_AX | STA_AX
        | STZ_AX => 4 + a + pc,
        // 6b Absolute, X(R-M-W) a,x
        ASL_AX | DEC_AX | INC_AX | LSR_AX | ROL_AX | ROR_AX => 7 + 2 * a,
        // 7. Absolute, Y a,y
        ADC_AY | AND_AY | CMP_AY | EOR_AY | LDA_AY | LDX_AY | ORA_AY | SBC_AY | STA_AY => {
            4 + a + pc
        }
        // 8. Accumulator A
        ASL_ACC | DEC_ACC | INC_ACC | LSR_ACC | ROL_ACC | ROR_ACC => 2,
        // 9a. Block Move Negative
        // 9b. Block Move Positive
        MVN | MVP => 7,
        // 10a. Direct d
        ADC_D | AND_D | BIT_D | CMP_D | CPX_D | CPY_D | EOR_D | LDA_D | LDX_D | LDY_D | ORA_D
        | SBC_D | STA_D | STX_D | STY_D | STZ_D => 3 + a + d,
        // 10b. Direct (R-M-W)d
        ASL_D | DEC_D | INC_D | LSR_D | ROL_D | ROR_D | TRB_D | TSB_D => 5 + 2 * a + d,
        // 11. Direct Indexed Indirect (d,x)
        ADC_DIX | AND_DIX | CMP_DIX | EOR_DIX | LDA_DIX | ORA_DIX | SBC_DIX | STA_DIX => 6 + a + d,
        // 12. Direct Indirect (d)
        ADC_DI | AND_DI | CMP_DI | EOR_DI | LDA_DI | ORA_DI | SBC_DI | STA_DI => 5 + a + d,
        // 13. Direct Indirect Indexed (d),y
        ADC_DIY | AND_DIY | CMP_DIY | EOR_DIY | LDA_DIY | ORA_DIY | SBC_DIY | STA_DIY => {
            5 + a + d + pc
        }
        // 14. Direct Indirect Indexed Long [d],y
        ADC_DILY | AND_DILY | CMP_DILY | EOR_DILY | LDA_DILY | ORA_DILY | SBC_DILY | STA_DILY => {
            6 + a + d
        }
        // 15. Direct Indirect Long [d]
        ADC_DIL | AND_DIL | CMP_DIL | EOR_DIL | LDA_DIL | ORA_DIL | SBC_DIL | STA_DIL => 6 + a + d,
        // 16a. Direct, X d,x
        ADC_DX | AND_DX | BIT_DX | CMP_DX | EOR_DX | LDA_DX | LDY_DX | ORA_DX | SBC_DX | STA_DX
        | STY_DX | STZ_DX => 4 + a + d,
        // 16b. Direct, X (R-M-W) d,x
        ASL_DX | DEC_DX | INC_DX | LSR_DX | ROL_DX | ROR_DX => 6 + 2 * a + d,
        // 17. Direct, Y d,y
        LDX_DY | STX_DY => 4 + a + d,
        // 18.Immediate #
        ADC_I | AND_I | BIT_I | CMP_I | CPX_I | CPY_I | EOR_I | LDA_I | LDX_I | LDY_I | ORA_I
        | SBC_I => 2 + a,
        REP_I | SEP_I => 3,
        // 19a. Implied i
        CLC | CLD | CLI | CLV | DEX | DEY | INX | INY | NOP | SEC | SED | SEI | TAX | TAY | TCD
        | TCS | TDC | TSC | TSX | TXA | TXS | TXY | TYA | TYX | XCE => 2,
        // 19b. Implied i
        XBA => 3,
        STP => unreachable!(),
        WAI => unreachable!(),
        // 20. Relative r (todo)
        BCC => 2 + usize::from(!p.p.c),
        BCS => 2 + usize::from(p.p.c),
        BEQ => 2 + usize::from(p.p.z),
        BNE => 2 + usize::from(!p.p.z),
        BMI => 2 + usize::from(p.p.n),
        BPL => 2 + usize::from(!p.p.n),
        BVC => 2 + usize::from(!p.p.v),
        BVS => 2 + usize::from(p.p.v),
        BRA => 3,
        // 21. Relative Long rl
        BRL => 4,
        // 22b. Stack s
        PLA | PLB | PLD | PLX | PLY | PLP => 4 + a,
        PHA | PHB | PHD | PHK | PHX | PHY => 3 + a,
        PHP => 3 + a,
        PEA => 5,
        PEI => 6 + d,
        PER => 6,
        RTI => 7 - e,
        RTS => 6,
        RTL => 6,
        BRK | COP => 8 - e,
        // 23. Stack Relative d,s
        ADC_SR | AND_SR | CMP_SR | EOR_SR | LDA_SR | ORA_SR | SBC_SR | STA_SR => 4 + a,
        // 24. Stack Relative Indirect Indexed (d,s),y
        ADC_SRIY | AND_SRIY | CMP_SRIY | EOR_SRIY | LDA_SRIY | ORA_SRIY | SBC_SRIY | STA_SRIY => {
            7 + a
        }
        _ => unreachable!("Missing timing information for {:02X}", opcode),
    }
}
macro_rules! timing_test {
    ($opc: ident, $nbits: expr) => {
        paste! {
            #[test]
            fn [<test_ $opc:lower _ $nbits bits>]() {
                let mut p = Processor::new();
                let mut memory = SimpleMemory::default();
                // Set opcode
                memory.memory[0] = $opc;
                // Execute
                p.step(&mut memory);
                // Assert counts
                let (r, w, io) = get_timings_for_opcode($opc, $nbits);
                assert_eq!(memory.num_reads, r);
                assert_eq!(memory.num_writes, w);
                assert_eq!(memory.num_io, io);
            }
        }
    };
}

#[test]
fn test_opcode_timings() {
    const SKIP_OPCODES: &[u8] = &[WDM, WAI, STP];
    (0..=0xFF).for_each(|opc| {
        if !SKIP_OPCODES.contains(&opc) {
            [8, 16].into_iter().for_each(|nbits| {
                let mut p = Processor::new();
                // Set 8 or 16 bit
                p.p.m = nbits == 8;
                p.p.xb = nbits == 8;
                let mut memory = SimpleMemory::default();
                // Set opcode
                memory.memory[0] = opc;
                // Execute
                p.step(&mut memory);
                // Assert counts
                let cycles = get_timings_for_opcode(opc, &p);
                assert_eq!(
                    cycles, memory.num_cycles,
                    "Number cycles for opcode {:02X}",
                    opc
                );
            })
        }
    })
}
