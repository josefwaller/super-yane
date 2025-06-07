/// Add with Carry Immediate
pub const ADC_I: u32 = 0x69;
/// Add with Carry Absolute
pub const ADC_A: u32 = 0x6D;
/// Add with Carry Absolute Long
pub const ADC_A_L: u32 = 0x6F;
/// Add with Carry Direct Page
pub const ADC_DP: u32 = 0x65;
/// Add with Carry Direct Page Indirect
pub const ADC_DP_I: u32 = 0x72;
/// Add with Carry Direct Page Indirect Long
pub const ADC_DP_IL: u32 = 0x67;
/// Add with Carry Absolute Indexed, X
pub const ADC_A_X: u32 = 0x7D;
/// Add with Carry Absolute Long Indexed, X
pub const ADC_A_LX: u32 = 0x7F;
/// Add with Carry Absolute Indexed, Y
pub const ADC_A_Y: u32 = 0x79;
/// Add with Carry Direct Page Indexed, X
pub const ADC_DP_X: u32 = 0x75;
/// Add with Carry Direct Page Indirect, X
pub const ADC_DP_IX: u32 = 0x61;
/// Add with Carry DP Indirect Indexed, Y
pub const ADC_DP_IY: u32 = 0x71;
/// Add with Carry DP Indirect Long Indexed, Y
pub const ADC_DP_ILY: u32 = 0x77;
/// Add with Carry Stack Relative
pub const ADC_SR: u32 = 0x63;
/// Add with Carry SR Indirect Indexed, Y
pub const ADC_SR_IY: u32 = 0x73;
/// And Accumulator with Memory Immediate
pub const AND_I: u32 = 0x29;
/// And Accumulator with Memory Absolute
pub const AND_A: u32 = 0x2D;
/// And Accumulator with Memory Absolute Long
pub const AND_A_L: u32 = 0x2F;
/// And Accumulator with Memory Direct Page
pub const AND_DP: u32 = 0x25;
/// And Accumulator with Memory Direct Page Indirect
pub const AND_DP_I: u32 = 0x32;
/// And Accumulator with Memory Direct Page Indirect Long
pub const AND_DP_IL: u32 = 0x27;
/// And Accumulator with Memory Absolute Indexed, X
pub const AND_A_X: u32 = 0x3D;
/// And Accumulator with Memory Absolute Long Indexed, X
pub const AND_A_LX: u32 = 0x3F;
/// And Accumulator with Memory Absolute Indexed, Y
pub const AND_A_Y: u32 = 0x39;
/// And Accumulator with Memory Direct Page Indexed, X
pub const AND_DP_X: u32 = 0x35;
/// And Accumulator with Memory Direct Page Indirect, X
pub const AND_DP_IX: u32 = 0x21;
/// And Accumulator with Memory DP Indirect Indexed, Y
pub const AND_DP_IY: u32 = 0x31;
/// And Accumulator with Memory DP Indirect Long Indexed, Y
pub const AND_DP_ILY: u32 = 0x37;
/// And Accumulator with Memory Stack Relative
pub const AND_SR: u32 = 0x23;
/// And Accumulator with Memory SR Indirect Indexed, Y
pub const AND_SR_IY: u32 = 0x33;
/// Arithmetic Shift Left Accumulator
pub const ASL_ACC: u32 = 0x0A;
/// Arithmetic Shift Left Absolute
pub const ASL_A: u32 = 0x0E;
/// Arithmetic Shift Left Direct Page
pub const ASL_DP: u32 = 0x06;
/// Arithmetic Shift Left Absolute Indexed, X
pub const ASL_A_X: u32 = 0x1E;
/// Arithmetic Shift Left Direct Page Indexed, X
pub const ASL_DP_X: u32 = 0x16;
/// Branch if Carry Clear
pub const BCC: u32 = 0x90;
/// Branch if Carry Set
pub const BCS: u32 = 0xB0;
/// Branch if Equal
pub const BEQ: u32 = 0xF0;
/// Test Memory Bits against Accumulator Immediate
pub const BIT_I: u32 = 0x89;
/// Test Memory Bits against Accumulator Absolute
pub const BIT_A: u32 = 0x2C;
/// Test Memory Bits against Accumulator Direct Page
pub const BIT_DP: u32 = 0x24;
/// Test Memory Bits against Accumulator Absolute Indexed, X
pub const BIT_A_X: u32 = 0x3C;
/// Test Memory Bits against Accumulator Direct Page Indexed, X
pub const BIT_DP_X: u32 = 0x34;
/// Branch if Minus
pub const BMI: u32 = 0x30;
/// Branch if Not Equal
pub const BNE: u32 = 0xD0;
/// Branch if Plus
pub const BPL: u32 = 0x10;
/// Branch Always
pub const BRA: u32 = 0x80;
/// BRK Software Interrupt
pub const BRK: u32 = 0x00;
/// Branch Always Long
pub const BRL: u32 = 0x82;
/// Branch if Overflow Clear
pub const BVC: u32 = 0x50;
/// Branch if Overflow Set
pub const BVS: u32 = 0x70;
/// Clear Carry Flag
pub const CLC: u32 = 0x18;
/// Clear Decimal Flag
pub const CLD: u32 = 0xD8;
/// Clear Interrupt Disable Flag
pub const CLI: u32 = 0x58;
/// Clear Overflow Flag
pub const CLV: u32 = 0xB8;
/// Compare Accumulator with Memory Immediate
pub const CMP_I: u32 = 0xC9;
/// Compare Accumulator with Memory Absolute
pub const CMP_A: u32 = 0xCD;
/// Compare Accumulator with Memory Absolute Long
pub const CMP_A_L: u32 = 0xCF;
/// Compare Accumulator with Memory Direct Page
pub const CMP_DP: u32 = 0xC5;
/// Compare Accumulator with Memory Direct Page Indirect
pub const CMP_DP_I: u32 = 0xD2;
/// Compare Accumulator with Memory Direct Page Indirect Long
pub const CMP_DP_IL: u32 = 0xC7;
/// Compare Accumulator with Memory Absolute Indexed, X
pub const CMP_A_X: u32 = 0xDD;
/// Compare Accumulator with Memory Absolute Long Indexed, X
pub const CMP_A_LX: u32 = 0xDF;
/// Compare Accumulator with Memory Absolute Indexed, Y
pub const CMP_A_Y: u32 = 0xD9;
/// Compare Accumulator with Memory Direct Page Indexed, X
pub const CMP_DP_X: u32 = 0xD5;
/// Compare Accumulator with Memory Direct Page Indirect, X
pub const CMP_DP_IX: u32 = 0xC1;
/// Compare Accumulator with Memory DP Indirect Indexed, Y
pub const CMP_DP_IY: u32 = 0xD1;
/// Compare Accumulator with Memory DP Indirect Long Indexed, Y
pub const CMP_DP_ILY: u32 = 0xD7;
/// Compare Accumulator with Memory Stack Relative
pub const CMP_SR: u32 = 0xC3;
/// Compare Accumulator with Memory SR Indirect Indexed, Y
pub const CMP_SR_IY: u32 = 0xD3;
/// COP Software Interrupt
pub const COP: u32 = 0x02;
/// Compare Index Register X with Memory Immediate
pub const CPX_I: u32 = 0xE0;
/// Compare Index Register X with Memory Absolute
pub const CPX_A: u32 = 0xEC;
/// Compare Index Register X with Memory Direct Page
pub const CPX_DP: u32 = 0xE4;
/// Compare Index Register Y with Memory Immediate
pub const CPY_I: u32 = 0xC0;
/// Compare Index Register Y with Memory Absolute
pub const CPY_A: u32 = 0xCC;
/// Compare Index Register Y with Memory Direct Page
pub const CPY_DP: u32 = 0xC4;
/// Decrement Accumulator
pub const DEC_ACC: u32 = 0x3A;
/// Decrement Absolute
pub const DEC_A: u32 = 0xCE;
/// Decrement Direct Page
pub const DEC_DP: u32 = 0xC6;
/// Decrement Absolute Indexed, X
pub const DEC_A_X: u32 = 0xDE;
/// Decrement Direct Page Indexed, X
pub const DEC_DP_X: u32 = 0xD6;
/// Decrement Index Registers Implied
pub const DEX: u32 = 0xCA;
/// Decrement Index Registers Implied
pub const DEY: u32 = 0x88;
/// Exclusive OR Accumulator with Memory Immediate
pub const EOR_I: u32 = 0x49;
/// Exclusive OR Accumulator with Memory Absolute
pub const EOR_A: u32 = 0x4D;
/// Exclusive OR Accumulator with Memory Absolute Long
pub const EOR_A_L: u32 = 0x4F;
/// Exclusive OR Accumulator with Memory Direct Page
pub const EOR_DP: u32 = 0x45;
/// Exclusive OR Accumulator with Memory Direct Page Indirect
pub const EOR_DP_I: u32 = 0x52;
/// Exclusive OR Accumulator with Memory Direct Page Indirect Long
pub const EOR_DP_IL: u32 = 0x47;
/// Exclusive OR Accumulator with Memory Absolute Indexed, X
pub const EOR_A_X: u32 = 0x5D;
/// Exclusive OR Accumulator with Memory Absolute Long Indexed, X
pub const EOR_A_LX: u32 = 0x5F;
/// Exclusive OR Accumulator with Memory Absolute Indexed, Y
pub const EOR_A_Y: u32 = 0x59;
/// Exclusive OR Accumulator with Memory Direct Page Indexed, X
pub const EOR_DP_X: u32 = 0x55;
/// Exclusive OR Accumulator with Memory Direct Page Indirect, X
pub const EOR_DP_IX: u32 = 0x41;
/// Exclusive OR Accumulator with Memory DP Indirect Indexed, Y
pub const EOR_DP_IY: u32 = 0x51;
/// Exclusive OR Accumulator with Memory DP Indirect Long Indexed, Y
pub const EOR_DP_ILY: u32 = 0x57;
/// Exclusive OR Accumulator with Memory Stack Relative
pub const EOR_SR: u32 = 0x43;
/// Exclusive OR Accumulator with Memory SR Indirect Indexed, Y
pub const EOR_SR_IY: u32 = 0x53;
/// Increment Accumulator
pub const INC_ACC: u32 = 0x1A;
/// Increment Absolute
pub const INC_A: u32 = 0xEE;
/// Increment Direct Page
pub const INC_DP: u32 = 0xE6;
/// Increment Absolute Indexed, X
pub const INC_A_X: u32 = 0xFE;
/// Increment Direct Page Indexed, X
pub const INC_DP_X: u32 = 0xF6;
/// Increment Index Registers Implied
pub const INX: u32 = 0xE8;
/// Increment Index Registers Implied
pub const INY: u32 = 0xC8;
/// Jump Absolute
pub const JMP_A: u32 = 0x4C;
/// Jump Absolute Indirect
pub const JMP_A_I: u32 = 0x6C;
/// Jump Absolute Indexed Indirect, X
pub const JMP_A_IX: u32 = 0x7C;
/// Jump Absolute Long
pub const JMP_A_L: u32 = 0x5C;
/// Jump Absolute Indirect Long
pub const JMP_A_IL: u32 = 0xDC;
/// Jump to Subroutine Absolute
pub const JSR_A: u32 = 0x20;
/// Jump to Subroutine Absolute Indexed Indirect, X
pub const JSR_A_IX: u32 = 0xFC;
/// Jump to Subroutine Absolute Long
pub const JSR_A_L: u32 = 0x22;
/// Load Accumulator from Memory Immediate
pub const LDA_I: u32 = 0xA9;
/// Load Accumulator from Memory Absolute
pub const LDA_A: u32 = 0xAD;
/// Load Accumulator from Memory Absolute Long
pub const LDA_A_L: u32 = 0xAF;
/// Load Accumulator from Memory Direct Page
pub const LDA_DP: u32 = 0xA5;
/// Load Accumulator from Memory Direct Page Indirect
pub const LDA_DP_I: u32 = 0xB2;
/// Load Accumulator from Memory Direct Page Indirect Long
pub const LDA_DP_IL: u32 = 0xA7;
/// Load Accumulator from Memory Absolute Indexed, X
pub const LDA_A_X: u32 = 0xBD;
/// Load Accumulator from Memory Absolute Long Indexed, X
pub const LDA_A_LX: u32 = 0xBF;
/// Load Accumulator from Memory Absolute Indexed, Y
pub const LDA_A_Y: u32 = 0xB9;
/// Load Accumulator from Memory Direct Page Indexed, X
pub const LDA_DP_X: u32 = 0xB5;
/// Load Accumulator from Memory Direct Page Indirect, X
pub const LDA_DP_IX: u32 = 0xA1;
/// Load Accumulator from Memory DP Indirect Indexed, Y
pub const LDA_DP_IY: u32 = 0xB1;
/// Load Accumulator from Memory DP Indirect Long Indexed, Y
pub const LDA_DP_ILY: u32 = 0xB7;
/// Load Accumulator from Memory Stack Relative
pub const LDA_SR: u32 = 0xA3;
/// Load Accumulator from Memory SR Indirect Indexed, Y
pub const LDA_SR_IY: u32 = 0xB3;
/// Load Index Register X from Memory Immediate
pub const LDX_I: u32 = 0xA2;
/// Load Index Register X from Memory Absolute
pub const LDX_A: u32 = 0xAE;
/// Load Index Register X from Memory Direct Page
pub const LDX_DP: u32 = 0xA6;
/// Load Index Register X from Memory Absolute Indexed, Y
pub const LDX_A_Y: u32 = 0xBE;
/// Load Index Register X from Memory Direct Page Indexed, Y
pub const LDX_DP_Y: u32 = 0xB6;
/// Load Index Register Y from Memory Immediate
pub const LDY_I: u32 = 0xA0;
/// Load Index Register Y from Memory Absolute
pub const LDY_A: u32 = 0xAC;
/// Load Index Register Y from Memory Direct Page
pub const LDY_DP: u32 = 0xA4;
/// Load Index Register Y from Memory Absolute Indexed, X
pub const LDY_A_X: u32 = 0xBC;
/// Load Index Register Y from Memory Direct Page Indexed, X
pub const LDY_DP_X: u32 = 0xB4;
/// Logical Shift Right Accumulator
pub const LSR_ACC: u32 = 0x4A;
/// Logical Shift Right Absolute
pub const LSR_A: u32 = 0x4E;
/// Logical Shift Right Direct Page
pub const LSR_DP: u32 = 0x46;
/// Logical Shift Right Absolute Indexed, X
pub const LSR_A_X: u32 = 0x5E;
/// Logical Shift Right Direct Page Indexed, X
pub const LSR_DP_X: u32 = 0x56;
/// Block Move Next
pub const MVN_NEXT: u32 = 0x54;
/// Block Move Previous
pub const MVN_PREV: u32 = 0x44;
/// No Operation Implied
pub const NOP: u32 = 0xEA;
/// OR Accumulator with Memory Immediate
pub const ORA_I: u32 = 0x09;
/// OR Accumulator with Memory Absolute
pub const ORA_A: u32 = 0x0D;
/// OR Accumulator with Memory Absolute Long
pub const ORA_A_L: u32 = 0x0F;
/// OR Accumulator with Memory Direct Page
pub const ORA_DP: u32 = 0x05;
/// OR Accumulator with Memory Direct Page Indirect
pub const ORA_DP_I: u32 = 0x12;
/// OR Accumulator with Memory Direct Page Indirect Long
pub const ORA_DP_IL: u32 = 0x07;
/// OR Accumulator with Memory Absolute Indexed, X
pub const ORA_A_X: u32 = 0x1D;
/// OR Accumulator with Memory Absolute Long Indexed, X
pub const ORA_A_LX: u32 = 0x1F;
/// OR Accumulator with Memory Absolute Indexed, Y
pub const ORA_A_Y: u32 = 0x19;
/// OR Accumulator with Memory Direct Page Indexed, X
pub const ORA_DP_X: u32 = 0x15;
/// OR Accumulator with Memory Direct Page Indirect, X
pub const ORA_DP_IX: u32 = 0x01;
/// OR Accumulator with Memory DP Indirect Indexed, Y
pub const ORA_DP_IY: u32 = 0x11;
/// OR Accumulator with Memory DP Indirect Long Indexed, Y
pub const ORA_DP_ILY: u32 = 0x17;
/// OR Accumulator with Memory Stack Relative
pub const ORA_SR: u32 = 0x03;
/// OR Accumulator with Memory SR Indirect Indexed, Y
pub const ORA_SR_IY: u32 = 0x13;
///  Push Effective Absolute Address
pub const PEA: u32 = 0xF4;
///  Push Effective Indirect Address
pub const PEI: u32 = 0xD4;
///  Push Effective PC Relative Indirect Address
pub const PER: u32 = 0x62;
/// Push Accumulator
pub const PHA: u32 = 0x48;
/// Push Data Bank
pub const PHB: u32 = 0x8B;
/// Push Direct Page Register
pub const PHD: u32 = 0x0B;
/// Push Program Bank Register
pub const PHK: u32 = 0x4B;
/// Push Processor Status Register
pub const PHP: u32 = 0x08;
/// Push Index Register X
pub const PHX: u32 = 0xDA;
/// Push Index Register Y
pub const PHY: u32 = 0x5A;
/// Pull Accumulator
pub const PLA: u32 = 0x68;
/// Pull Data Bank
pub const PLB: u32 = 0xAB;
/// Pull Direct Page Register
pub const PLD: u32 = 0x2B;
/// Pull Processor Status Register
pub const PLP: u32 = 0x28;
/// Pull Index Register X
pub const PLX: u32 = 0xFA;
/// Pull Index Register Y
pub const PLY: u32 = 0x7A;
/// Reset Status Bits Immediate
pub const REP_I: u32 = 0xC2;
/// Rotate Left Accumulator
pub const ROL_ACC: u32 = 0x2A;
/// Rotate Left Absolute
pub const ROL_A: u32 = 0x2E;
/// Rotate Left Direct Page
pub const ROL_DP: u32 = 0x26;
/// Rotate Left Absolute Indexed, X
pub const ROL_A_X: u32 = 0x3E;
/// Rotate Left Direct Page Indexed, X
pub const ROL_DP_X: u32 = 0x36;
/// Rotate Right Accumulator
pub const ROR_ACC: u32 = 0x6A;
/// Rotate Right Absolute
pub const ROR_A: u32 = 0x6E;
/// Rotate Right Direct Page
pub const ROR_DP: u32 = 0x66;
/// Rotate Right Absolute Indexed, X
pub const ROR_A_X: u32 = 0x7E;
/// Rotate Right Direct Page Indexed, X
pub const ROR_DP_X: u32 = 0x76;
///  Return From Interrupt
pub const RTI: u32 = 0x40;
/// Return From Subroutine Long
pub const RTL: u32 = 0x6B;
///  Return From Subroutine
pub const RTS: u32 = 0x60;
/// Subtract with Borrow from Accumulator Immediate
pub const SBC_I: u32 = 0xE9;
/// Subtract with Borrow from Accumulator Absolute
pub const SBC_A: u32 = 0xED;
/// Subtract with Borrow from Accumulator Absolute Long
pub const SBC_A_L: u32 = 0xEF;
/// Subtract with Borrow from Accumulator Direct Page
pub const SBC_DP: u32 = 0xE5;
/// Subtract with Borrow from Accumulator Direct Page Indirect
pub const SBC_DP_I: u32 = 0xF2;
/// Subtract with Borrow from Accumulator Direct Page Indirect Long
pub const SBC_DP_IL: u32 = 0xE7;
/// Subtract with Borrow from Accumulator Absolute Indexed, X
pub const SBC_A_X: u32 = 0xFD;
/// Subtract with Borrow from Accumulator Absolute Long Indexed, X
pub const SBC_A_LX: u32 = 0xFF;
/// Subtract with Borrow from Accumulator Absolute Indexed, Y
pub const SBC_A_Y: u32 = 0xF9;
/// Subtract with Borrow from Accumulator Direct Page Indexed, X
pub const SBC_DP_X: u32 = 0xF5;
/// Subtract with Borrow from Accumulator Direct Page Indirect, X
pub const SBC_DP_IX: u32 = 0xE1;
/// Subtract with Borrow from Accumulator DP Indirect Indexed, Y
pub const SBC_DP_IY: u32 = 0xF1;
/// Subtract with Borrow from Accumulator DP Indirect Long Indexed, Y
pub const SBC_DP_ILY: u32 = 0xF7;
/// Subtract with Borrow from Accumulator Stack Relative
pub const SBC_SR: u32 = 0xE3;
/// Subtract with Borrow from Accumulator SR Indirect Indexed, Y
pub const SBC_SR_IY: u32 = 0xF3;
/// Set Carry Flag
pub const SEC: u32 = 0x38;
/// Set Decimal Flag
pub const SED: u32 = 0xF8;
/// Set Interrupt Disable Flag
pub const SEI: u32 = 0x78;
/// Set Status Bits Immediate
pub const SEP_I: u32 = 0xE2;
/// Store Accumulator to Memory Absolute
pub const STA_A: u32 = 0x8D;
/// Store Accumulator to Memory Absolute Long
pub const STA_A_L: u32 = 0x8F;
/// Store Accumulator to Memory Direct Page
pub const STA_DP: u32 = 0x85;
/// Store Accumulator to Memory Direct Page Indirect
pub const STA_DP_I: u32 = 0x92;
/// Store Accumulator to Memory Direct Page Indirect Long
pub const STA_DP_IL: u32 = 0x87;
/// Store Accumulator to Memory Absolute Indexed, X
pub const STA_A_X: u32 = 0x9D;
/// Store Accumulator to Memory Absolute Long Indexed, X
pub const STA_A_LX: u32 = 0x9F;
/// Store Accumulator to Memory Absolute Indexed, Y
pub const STA_A_Y: u32 = 0x99;
/// Store Accumulator to Memory Direct Page Indexed, X
pub const STA_DP_X: u32 = 0x95;
/// Store Accumulator to Memory Direct Page Indirect, X
pub const STA_DP_IX: u32 = 0x81;
/// Store Accumulator to Memory DP Indirect Indexed, Y
pub const STA_DP_IY: u32 = 0x91;
/// Store Accumulator to Memory DP Indirect Long Indexed, Y
pub const STA_DP_ILY: u32 = 0x97;
/// Store Accumulator to Memory Stack Relative
pub const STA_SR: u32 = 0x83;
/// Store Accumulator to Memory SR Indirect Indexed, Y
pub const STA_SR_IY: u32 = 0x93;
/// Stop the Processor Implied
pub const STP: u32 = 0xDB;
/// Store Index Register X to Memory Absolute
pub const STX_A: u32 = 0x8E;
/// Store Index Register X to Memory Direct Page
pub const STX_DP: u32 = 0x86;
/// Store Index Register X to Memory Direct Page Indexed, Y
pub const STX_DP_Y: u32 = 0x96;
/// Store Index Register Y to Memory Absolute
pub const STY_A: u32 = 0x8C;
/// Store Index Register Y to Memory Direct Page
pub const STY_DP: u32 = 0x84;
/// Store Index Register Y to Memory Direct Page Indexed, X
pub const STY_DP_X: u32 = 0x94;
/// Store Zero to Memory Absolute
pub const STZ_A: u32 = 0x9C;
/// Store Zero to Memory Direct Page
pub const STZ_DP: u32 = 0x64;
/// Store Zero to Memory Absolute Indexed, X
pub const STZ_A_X: u32 = 0x9E;
/// Store Zero to Memory Direct Page Indexed, X
pub const STZ_DP_X: u32 = 0x74;
/// Transfer A to X
pub const TAX: u32 = 0xAA;
/// Transfer A to Y
pub const TAY: u32 = 0xA8;
/// Transfer 16 bit A to D
pub const TCD: u32 = 0x5B;
/// Transfer 16 bit A to S
pub const TCS: u32 = 0x1B;
/// Transfer D to 16 bit A
pub const TDC: u32 = 0x7B;
/// Test and Reset Memory Bits Against Accumulator Absolute
pub const TRB_A: u32 = 0x1C;
/// Test and Reset Memory Bits Against Accumulator Direct Page
pub const TRB_DP: u32 = 0x14;
/// Test and Set Memory Bits Against Accumulator Absolute
pub const TSB_A: u32 = 0x0C;
/// Test and Set Memory Bits Against Accumulator Direct Page
pub const TSB_DP: u32 = 0x04;
/// Transfer S to 16 bit A
pub const TSC: u32 = 0x3B;
/// Transfer S to X
pub const TSX: u32 = 0xBA;
/// Transfer X to A
pub const TXA: u32 = 0x8A;
/// Transfer X to S
pub const TXS: u32 = 0x9A;
/// Transfer X to Y
pub const TXY: u32 = 0x9B;
/// Transfer Y to A
pub const TYA: u32 = 0x98;
/// Transfer Y to X
pub const TYX: u32 = 0xBB;
/// Wait for Interrupt Implied
pub const WAI: u32 = 0xCB;
/// Reserved for Future Expansion
pub const WDM: u32 = 0x42;
/// Exchange the B and A Accumulators Implied
pub const XBA: u32 = 0xEB;
/// Exchange Carry and Emulation Bits Implied
pub const XCE: u32 = 0xFB;
