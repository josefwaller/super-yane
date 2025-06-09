/// Add with Carry Immediate
pub const ADC_I: u8 = 0x69;
/// Add with Carry Absolute
pub const ADC_A: u8 = 0x6D;
/// Add with Carry Absolute Long
pub const ADC_AL: u8 = 0x6F;
/// Add with Carry Direct
pub const ADC_D: u8 = 0x65;
/// Add with Carry Direct Indirect
pub const ADC_DI: u8 = 0x72;
/// Add with Carry Direct Indirect Long
pub const ADC_DIL: u8 = 0x67;
/// Add with Carry Absolute Indexed, X
pub const ADC_AX: u8 = 0x7D;
/// Add with Carry Absolute Long Indexed, X
pub const ADC_ALX: u8 = 0x7F;
/// Add with Carry Absolute Indexed, Y
pub const ADC_AY: u8 = 0x79;
/// Add with Carry Direct Indexed, X
pub const ADC_DX: u8 = 0x75;
/// Add with Carry Direct Indirect, X
pub const ADC_DIX: u8 = 0x61;
/// Add with Carry D Indirect Indexed, Y
pub const ADC_DIY: u8 = 0x71;
/// Add with Carry D Indirect Long Indexed, Y
pub const ADC_DILY: u8 = 0x77;
/// Add with Carry Stack Relative
pub const ADC_SR: u8 = 0x63;
/// Add with Carry SR Indirect Indexed, Y
pub const ADC_SRIY: u8 = 0x73;
/// And Accumulator with Memory Immediate
pub const AND_I: u8 = 0x29;
/// And Accumulator with Memory Absolute
pub const AND_A: u8 = 0x2D;
/// And Accumulator with Memory Absolute Long
pub const AND_AL: u8 = 0x2F;
/// And Accumulator with Memory Direct
pub const AND_D: u8 = 0x25;
/// And Accumulator with Memory Direct Indirect
pub const AND_DI: u8 = 0x32;
/// And Accumulator with Memory Direct Indirect Long
pub const AND_DIL: u8 = 0x27;
/// And Accumulator with Memory Absolute Indexed, X
pub const AND_AX: u8 = 0x3D;
/// And Accumulator with Memory Absolute Long Indexed, X
pub const AND_ALX: u8 = 0x3F;
/// And Accumulator with Memory Absolute Indexed, Y
pub const AND_AY: u8 = 0x39;
/// And Accumulator with Memory Direct Indexed, X
pub const AND_DX: u8 = 0x35;
/// And Accumulator with Memory Direct Indirect, X
pub const AND_DIX: u8 = 0x21;
/// And Accumulator with Memory D Indirect Indexed, Y
pub const AND_DIY: u8 = 0x31;
/// And Accumulator with Memory D Indirect Long Indexed, Y
pub const AND_DILY: u8 = 0x37;
/// And Accumulator with Memory Stack Relative
pub const AND_SR: u8 = 0x23;
/// And Accumulator with Memory SR Indirect Indexed, Y
pub const AND_SRIY: u8 = 0x33;
/// Arithmetic Shift Left Accumulator
pub const ASL_ACC: u8 = 0x0A;
/// Arithmetic Shift Left Absolute
pub const ASL_A: u8 = 0x0E;
/// Arithmetic Shift Left Direct
pub const ASL_D: u8 = 0x06;
/// Arithmetic Shift Left Absolute Indexed, X
pub const ASL_AX: u8 = 0x1E;
/// Arithmetic Shift Left Direct Indexed, X
pub const ASL_DX: u8 = 0x16;
/// Branch if Carry Clear
pub const BCC: u8 = 0x90;
/// Branch if Carry Set
pub const BCS: u8 = 0xB0;
/// Branch if Equal
pub const BEQ: u8 = 0xF0;
/// Test Memory Bits against Accumulator Immediate
pub const BIT_I: u8 = 0x89;
/// Test Memory Bits against Accumulator Absolute
pub const BIT_A: u8 = 0x2C;
/// Test Memory Bits against Accumulator Direct
pub const BIT_D: u8 = 0x24;
/// Test Memory Bits against Accumulator Absolute Indexed, X
pub const BIT_AX: u8 = 0x3C;
/// Test Memory Bits against Accumulator Direct Indexed, X
pub const BIT_DX: u8 = 0x34;
/// Branch if Minus
pub const BMI: u8 = 0x30;
/// Branch if Not Equal
pub const BNE: u8 = 0xD0;
/// Branch if Plus
pub const BPL: u8 = 0x10;
/// Branch Always
pub const BRA: u8 = 0x80;
/// BRK Software Interrupt
pub const BRK: u8 = 0x00;
/// Branch Always Long
pub const BRL: u8 = 0x82;
/// Branch if Overflow Clear
pub const BVC: u8 = 0x50;
/// Branch if Overflow Set
pub const BVS: u8 = 0x70;
/// Clear Carry Flag
pub const CLC: u8 = 0x18;
/// Clear Decimal Flag
pub const CLD: u8 = 0xD8;
/// Clear Interrupt Disable Flag
pub const CLI: u8 = 0x58;
/// Clear Overflow Flag
pub const CLV: u8 = 0xB8;
/// Compare Accumulator with Memory Immediate
pub const CMP_I: u8 = 0xC9;
/// Compare Accumulator with Memory Absolute
pub const CMP_A: u8 = 0xCD;
/// Compare Accumulator with Memory Absolute Long
pub const CMP_AL: u8 = 0xCF;
/// Compare Accumulator with Memory Direct
pub const CMP_D: u8 = 0xC5;
/// Compare Accumulator with Memory Direct Indirect
pub const CMP_DI: u8 = 0xD2;
/// Compare Accumulator with Memory Direct Indirect Long
pub const CMP_DIL: u8 = 0xC7;
/// Compare Accumulator with Memory Absolute Indexed, X
pub const CMP_AX: u8 = 0xDD;
/// Compare Accumulator with Memory Absolute Long Indexed, X
pub const CMP_ALX: u8 = 0xDF;
/// Compare Accumulator with Memory Absolute Indexed, Y
pub const CMP_AY: u8 = 0xD9;
/// Compare Accumulator with Memory Direct Indexed, X
pub const CMP_DX: u8 = 0xD5;
/// Compare Accumulator with Memory Direct Indirect, X
pub const CMP_DIX: u8 = 0xC1;
/// Compare Accumulator with Memory D Indirect Indexed, Y
pub const CMP_DIY: u8 = 0xD1;
/// Compare Accumulator with Memory D Indirect Long Indexed, Y
pub const CMP_DILY: u8 = 0xD7;
/// Compare Accumulator with Memory Stack Relative
pub const CMP_SR: u8 = 0xC3;
/// Compare Accumulator with Memory SR Indirect Indexed, Y
pub const CMP_SRIY: u8 = 0xD3;
/// COP Software Interrupt
pub const COP: u8 = 0x02;
/// Compare Index Register X with Memory Immediate
pub const CPX_I: u8 = 0xE0;
/// Compare Index Register X with Memory Absolute
pub const CPX_A: u8 = 0xEC;
/// Compare Index Register X with Memory Direct
pub const CPX_D: u8 = 0xE4;
/// Compare Index Register Y with Memory Immediate
pub const CPY_I: u8 = 0xC0;
/// Compare Index Register Y with Memory Absolute
pub const CPY_A: u8 = 0xCC;
/// Compare Index Register Y with Memory Direct
pub const CPY_D: u8 = 0xC4;
/// Decrement Accumulator
pub const DEC_ACC: u8 = 0x3A;
/// Decrement Absolute
pub const DEC_A: u8 = 0xCE;
/// Decrement Direct
pub const DEC_D: u8 = 0xC6;
/// Decrement Absolute Indexed, X
pub const DEC_AX: u8 = 0xDE;
/// Decrement Direct Indexed, X
pub const DEC_DX: u8 = 0xD6;
/// Decrement X Implied
pub const DEX: u8 = 0xCA;
/// Decrement Y Implied
pub const DEY: u8 = 0x88;
/// Exclusive OR Accumulator with Memory Immediate
pub const EOR_I: u8 = 0x49;
/// Exclusive OR Accumulator with Memory Absolute
pub const EOR_A: u8 = 0x4D;
/// Exclusive OR Accumulator with Memory Absolute Long
pub const EOR_AL: u8 = 0x4F;
/// Exclusive OR Accumulator with Memory Direct
pub const EOR_D: u8 = 0x45;
/// Exclusive OR Accumulator with Memory Direct Indirect
pub const EOR_DI: u8 = 0x52;
/// Exclusive OR Accumulator with Memory Direct Indirect Long
pub const EOR_DIL: u8 = 0x47;
/// Exclusive OR Accumulator with Memory Absolute Indexed, X
pub const EOR_AX: u8 = 0x5D;
/// Exclusive OR Accumulator with Memory Absolute Long Indexed, X
pub const EOR_ALX: u8 = 0x5F;
/// Exclusive OR Accumulator with Memory Absolute Indexed, Y
pub const EOR_AY: u8 = 0x59;
/// Exclusive OR Accumulator with Memory Direct Indexed, X
pub const EOR_DX: u8 = 0x55;
/// Exclusive OR Accumulator with Memory Direct Indirect, X
pub const EOR_DIX: u8 = 0x41;
/// Exclusive OR Accumulator with Memory D Indirect Indexed, Y
pub const EOR_DIY: u8 = 0x51;
/// Exclusive OR Accumulator with Memory D Indirect Long Indexed, Y
pub const EOR_DILY: u8 = 0x57;
/// Exclusive OR Accumulator with Memory Stack Relative
pub const EOR_SR: u8 = 0x43;
/// Exclusive OR Accumulator with Memory SR Indirect Indexed, Y
pub const EOR_SRIY: u8 = 0x53;
/// Increment Accumulator
pub const INC_ACC: u8 = 0x1A;
/// Increment Absolute
pub const INC_A: u8 = 0xEE;
/// Increment Direct
pub const INC_D: u8 = 0xE6;
/// Increment Absolute Indexed, X
pub const INC_AX: u8 = 0xFE;
/// Increment Direct Indexed, X
pub const INC_DX: u8 = 0xF6;
/// Increment X Implied
pub const INX: u8 = 0xE8;
/// Increment Y Implied
pub const INY: u8 = 0xC8;
/// Jump Absolute
pub const JMP_A: u8 = 0x4C;
/// Jump Absolute Indirect
pub const JMP_AI: u8 = 0x6C;
/// Jump Absolute Indexed Indirect, X
pub const JMP_AIX: u8 = 0x7C;
/// Jump Absolute Long
pub const JMP_AL: u8 = 0x5C;
/// Jump Absolute Indirect Long
pub const JMP_AIL: u8 = 0xDC;
/// Jump to Subroutine Absolute
pub const JSR_A: u8 = 0x20;
/// Jump to Subroutine Absolute Indexed Indirect, X
pub const JSR_AIX: u8 = 0xFC;
/// Jump to Subroutine Absolute Long
pub const JSR_AL: u8 = 0x22;
/// Load Accumulator from Memory Immediate
pub const LDA_I: u8 = 0xA9;
/// Load Accumulator from Memory Absolute
pub const LDA_A: u8 = 0xAD;
/// Load Accumulator from Memory Absolute Long
pub const LDA_AL: u8 = 0xAF;
/// Load Accumulator from Memory Direct
pub const LDA_D: u8 = 0xA5;
/// Load Accumulator from Memory Direct Indirect
pub const LDA_DI: u8 = 0xB2;
/// Load Accumulator from Memory Direct Indirect Long
pub const LDA_DIL: u8 = 0xA7;
/// Load Accumulator from Memory Absolute Indexed, X
pub const LDA_AX: u8 = 0xBD;
/// Load Accumulator from Memory Absolute Long Indexed, X
pub const LDA_ALX: u8 = 0xBF;
/// Load Accumulator from Memory Absolute Indexed, Y
pub const LDA_AY: u8 = 0xB9;
/// Load Accumulator from Memory Direct Indexed, X
pub const LDA_DX: u8 = 0xB5;
/// Load Accumulator from Memory Direct Indirect, X
pub const LDA_DIX: u8 = 0xA1;
/// Load Accumulator from Memory D Indirect Indexed, Y
pub const LDA_DIY: u8 = 0xB1;
/// Load Accumulator from Memory D Indirect Long Indexed, Y
pub const LDA_DILY: u8 = 0xB7;
/// Load Accumulator from Memory Stack Relative
pub const LDA_SR: u8 = 0xA3;
/// Load Accumulator from Memory SR Indirect Indexed, Y
pub const LDA_SRIY: u8 = 0xB3;
/// Load Index Register X from Memory Immediate
pub const LDX_I: u8 = 0xA2;
/// Load Index Register X from Memory Absolute
pub const LDX_A: u8 = 0xAE;
/// Load Index Register X from Memory Direct
pub const LDX_D: u8 = 0xA6;
/// Load Index Register X from Memory Absolute Indexed, Y
pub const LDX_AY: u8 = 0xBE;
/// Load Index Register X from Memory Direct Indexed, Y
pub const LDX_DY: u8 = 0xB6;
/// Load Index Register Y from Memory Immediate
pub const LDY_I: u8 = 0xA0;
/// Load Index Register Y from Memory Absolute
pub const LDY_A: u8 = 0xAC;
/// Load Index Register Y from Memory Direct
pub const LDY_D: u8 = 0xA4;
/// Load Index Register Y from Memory Absolute Indexed, X
pub const LDY_AX: u8 = 0xBC;
/// Load Index Register Y from Memory Direct Indexed, X
pub const LDY_DX: u8 = 0xB4;
/// Logical Shift Right Accumulator
pub const LSR_ACC: u8 = 0x4A;
/// Logical Shift Right Absolute
pub const LSR_A: u8 = 0x4E;
/// Logical Shift Right Direct
pub const LSR_D: u8 = 0x46;
/// Logical Shift Right Absolute Indexed, X
pub const LSR_AX: u8 = 0x5E;
/// Logical Shift Right Direct Indexed, X
pub const LSR_DX: u8 = 0x56;
/// Block Move Next
pub const MVN_NEXT: u8 = 0x54;
/// Block Move Previous
pub const MVN_PREV: u8 = 0x44;
/// No Operation Implied
pub const NOP: u8 = 0xEA;
/// OR Accumulator with Memory Immediate
pub const ORA_I: u8 = 0x09;
/// OR Accumulator with Memory Absolute
pub const ORA_A: u8 = 0x0D;
/// OR Accumulator with Memory Absolute Long
pub const ORA_AL: u8 = 0x0F;
/// OR Accumulator with Memory Direct
pub const ORA_D: u8 = 0x05;
/// OR Accumulator with Memory Direct Indirect
pub const ORA_DI: u8 = 0x12;
/// OR Accumulator with Memory Direct Indirect Long
pub const ORA_DIL: u8 = 0x07;
/// OR Accumulator with Memory Absolute Indexed, X
pub const ORA_AX: u8 = 0x1D;
/// OR Accumulator with Memory Absolute Long Indexed, X
pub const ORA_ALX: u8 = 0x1F;
/// OR Accumulator with Memory Absolute Indexed, Y
pub const ORA_AY: u8 = 0x19;
/// OR Accumulator with Memory Direct Indexed, X
pub const ORA_DX: u8 = 0x15;
/// OR Accumulator with Memory Direct Indirect, X
pub const ORA_DIX: u8 = 0x01;
/// OR Accumulator with Memory D Indirect Indexed, Y
pub const ORA_DIY: u8 = 0x11;
/// OR Accumulator with Memory D Indirect Long Indexed, Y
pub const ORA_DILY: u8 = 0x17;
/// OR Accumulator with Memory Stack Relative
pub const ORA_SR: u8 = 0x03;
/// OR Accumulator with Memory SR Indirect Indexed, Y
pub const ORA_SRIY: u8 = 0x13;
///  Push Effective Absolute Address
pub const PEA: u8 = 0xF4;
///  Push Effective Indirect Address
pub const PEI: u8 = 0xD4;
///  Push Effective PC Relative Indirect Address
pub const PER: u8 = 0x62;
/// Push Accumulator
pub const PHA: u8 = 0x48;
/// Push Data Bank
pub const PHB: u8 = 0x8B;
/// Push Direct Register
pub const PHD: u8 = 0x0B;
/// Push Program Bank Register
pub const PHK: u8 = 0x4B;
/// Push Processor Status Register
pub const PHP: u8 = 0x08;
/// Push Index Register X
pub const PHX: u8 = 0xDA;
/// Push Index Register Y
pub const PHY: u8 = 0x5A;
/// Pull Accumulator
pub const PLA: u8 = 0x68;
/// Pull Data Bank
pub const PLB: u8 = 0xAB;
/// Pull Direct Register
pub const PLD: u8 = 0x2B;
/// Pull Processor Status Register
pub const PLP: u8 = 0x28;
/// Pull Index Register X
pub const PLX: u8 = 0xFA;
/// Pull Index Register Y
pub const PLY: u8 = 0x7A;
/// Reset Status Bits Immediate
pub const REP_I: u8 = 0xC2;
/// Rotate Left Accumulator
pub const ROL_ACC: u8 = 0x2A;
/// Rotate Left Absolute
pub const ROL_A: u8 = 0x2E;
/// Rotate Left Direct
pub const ROL_D: u8 = 0x26;
/// Rotate Left Absolute Indexed, X
pub const ROL_AX: u8 = 0x3E;
/// Rotate Left Direct Indexed, X
pub const ROL_DX: u8 = 0x36;
/// Rotate Right Accumulator
pub const ROR_ACC: u8 = 0x6A;
/// Rotate Right Absolute
pub const ROR_A: u8 = 0x6E;
/// Rotate Right Direct
pub const ROR_D: u8 = 0x66;
/// Rotate Right Absolute Indexed, X
pub const ROR_AX: u8 = 0x7E;
/// Rotate Right Direct Indexed, X
pub const ROR_DX: u8 = 0x76;
///  Return From Interrupt
pub const RTI: u8 = 0x40;
/// Return From Subroutine Long
pub const RTL: u8 = 0x6B;
///  Return From Subroutine
pub const RTS: u8 = 0x60;
/// Subtract with Borrow from Accumulator Immediate
pub const SBC_I: u8 = 0xE9;
/// Subtract with Borrow from Accumulator Absolute
pub const SBC_A: u8 = 0xED;
/// Subtract with Borrow from Accumulator Absolute Long
pub const SBC_AL: u8 = 0xEF;
/// Subtract with Borrow from Accumulator Direct
pub const SBC_D: u8 = 0xE5;
/// Subtract with Borrow from Accumulator Direct Indirect
pub const SBC_DI: u8 = 0xF2;
/// Subtract with Borrow from Accumulator Direct Indirect Long
pub const SBC_DIL: u8 = 0xE7;
/// Subtract with Borrow from Accumulator Absolute Indexed, X
pub const SBC_AX: u8 = 0xFD;
/// Subtract with Borrow from Accumulator Absolute Long Indexed, X
pub const SBC_ALX: u8 = 0xFF;
/// Subtract with Borrow from Accumulator Absolute Indexed, Y
pub const SBC_AY: u8 = 0xF9;
/// Subtract with Borrow from Accumulator Direct Indexed, X
pub const SBC_DX: u8 = 0xF5;
/// Subtract with Borrow from Accumulator Direct Indirect, X
pub const SBC_DIX: u8 = 0xE1;
/// Subtract with Borrow from Accumulator D Indirect Indexed, Y
pub const SBC_DIY: u8 = 0xF1;
/// Subtract with Borrow from Accumulator D Indirect Long Indexed, Y
pub const SBC_DILY: u8 = 0xF7;
/// Subtract with Borrow from Accumulator Stack Relative
pub const SBC_SR: u8 = 0xE3;
/// Subtract with Borrow from Accumulator SR Indirect Indexed, Y
pub const SBC_SRIY: u8 = 0xF3;
/// Set Carry Flag
pub const SEC: u8 = 0x38;
/// Set Decimal Flag
pub const SED: u8 = 0xF8;
/// Set Interrupt Disable Flag
pub const SEI: u8 = 0x78;
/// Set Status Bits Immediate
pub const SEP_I: u8 = 0xE2;
/// Store Accumulator to Memory Absolute
pub const STA_A: u8 = 0x8D;
/// Store Accumulator to Memory Absolute Long
pub const STA_AL: u8 = 0x8F;
/// Store Accumulator to Memory Direct
pub const STA_D: u8 = 0x85;
/// Store Accumulator to Memory Direct Indirect
pub const STA_DI: u8 = 0x92;
/// Store Accumulator to Memory Direct Indirect Long
pub const STA_DIL: u8 = 0x87;
/// Store Accumulator to Memory Absolute Indexed, X
pub const STA_AX: u8 = 0x9D;
/// Store Accumulator to Memory Absolute Long Indexed, X
pub const STA_ALX: u8 = 0x9F;
/// Store Accumulator to Memory Absolute Indexed, Y
pub const STA_AY: u8 = 0x99;
/// Store Accumulator to Memory Direct Indexed, X
pub const STA_DX: u8 = 0x95;
/// Store Accumulator to Memory Direct Indirect, X
pub const STA_DIX: u8 = 0x81;
/// Store Accumulator to Memory D Indirect Indexed, Y
pub const STA_DIY: u8 = 0x91;
/// Store Accumulator to Memory D Indirect Long Indexed, Y
pub const STA_DILY: u8 = 0x97;
/// Store Accumulator to Memory Stack Relative
pub const STA_SR: u8 = 0x83;
/// Store Accumulator to Memory SR Indirect Indexed, Y
pub const STA_SRIY: u8 = 0x93;
/// Stop the Processor Implied
pub const STP: u8 = 0xDB;
/// Store Index Register X to Memory Absolute
pub const STX_A: u8 = 0x8E;
/// Store Index Register X to Memory Direct
pub const STX_D: u8 = 0x86;
/// Store Index Register X to Memory Direct Indexed, Y
pub const STX_DY: u8 = 0x96;
/// Store Index Register Y to Memory Absolute
pub const STY_A: u8 = 0x8C;
/// Store Index Register Y to Memory Direct
pub const STY_D: u8 = 0x84;
/// Store Index Register Y to Memory Direct Indexed, X
pub const STY_DX: u8 = 0x94;
/// Store Zero to Memory Absolute
pub const STZ_A: u8 = 0x9C;
/// Store Zero to Memory Direct
pub const STZ_D: u8 = 0x64;
/// Store Zero to Memory Absolute Indexed, X
pub const STZ_AX: u8 = 0x9E;
/// Store Zero to Memory Direct Indexed, X
pub const STZ_DX: u8 = 0x74;
/// Transfer A to X
pub const TAX: u8 = 0xAA;
/// Transfer A to Y
pub const TAY: u8 = 0xA8;
/// Transfer 16 bit A to D
pub const TCD: u8 = 0x5B;
/// Transfer 16 bit A to S
pub const TCS: u8 = 0x1B;
/// Transfer D to 16 bit A
pub const TDC: u8 = 0x7B;
/// Test and Reset Memory Bits Against Accumulator Absolute
pub const TRB_A: u8 = 0x1C;
/// Test and Reset Memory Bits Against Accumulator Direct
pub const TRB_D: u8 = 0x14;
/// Test and Set Memory Bits Against Accumulator Absolute
pub const TSB_A: u8 = 0x0C;
/// Test and Set Memory Bits Against Accumulator Direct
pub const TSB_D: u8 = 0x04;
/// Transfer S to 16 bit A
pub const TSC: u8 = 0x3B;
/// Transfer S to X
pub const TSX: u8 = 0xBA;
/// Transfer X to A
pub const TXA: u8 = 0x8A;
/// Transfer X to S
pub const TXS: u8 = 0x9A;
/// Transfer X to Y
pub const TXY: u8 = 0x9B;
/// Transfer Y to A
pub const TYA: u8 = 0x98;
/// Transfer Y to X
pub const TYX: u8 = 0xBB;
/// Wait for Interrupt Implied
pub const WAI: u8 = 0xCB;
/// Reserved for Future Expansion
pub const WDM: u8 = 0x42;
/// Exchange the B and A Accumulators Implied
pub const XBA: u8 = 0xEB;
/// Exchange Carry and Emulation Bits Implied
pub const XCE: u8 = 0xFB;
