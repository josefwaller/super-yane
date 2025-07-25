/// Add with carry X Indexed Y Indexed
pub const ADC_IX_IY: u8 = 0x99;
/// Add with carry A Register Immediate
pub const ADC_A_IMM: u8 = 0x88;
/// Add with carry A Register X Indexed
pub const ADC_A_IX: u8 = 0x86;
/// Add with carry A Register Y Indexed Indirect Direct Page
pub const ADC_A_IDY: u8 = 0x97;
/// Add with carry A Register Indirect X Indexed Direct Page
pub const ADC_A_IDX: u8 = 0x87;
/// Add with carry A Register Direct Page
pub const ADC_A_D: u8 = 0x84;
/// Add with carry A Register Direct Page X Indexed
pub const ADC_A_DX: u8 = 0x94;
/// Add with carry A Register Absolute
pub const ADC_A_ABS: u8 = 0x85;
/// Add with carry A Register Absolute X Indexed
pub const ADC_A_ABSX: u8 = 0x95;
/// Add with carry A Register Absolute Y Indexed
pub const ADC_A_ABSY: u8 = 0x96;
/// Add with carry Direct Page Direct Page
pub const ADC_D_D: u8 = 0x89;
/// Add with carry Direct Page Immediate
pub const ADC_D_IMM: u8 = 0x98;
/// Add word Y Register and A Register Combined Direct Page
pub const ADDW_YA_D: u8 = 0x7A;
/// Logical AND X Indexed Y Indexed
pub const AND_IX_IY: u8 = 0x39;
/// Logical AND A Register Immediate
pub const AND_A_IMM: u8 = 0x28;
/// Logical AND A Register X Indexed
pub const AND_A_IX: u8 = 0x26;
/// Logical AND A Register Y Indexed Indirect Direct Page
pub const AND_A_IDY: u8 = 0x37;
/// Logical AND A Register Indirect X Indexed Direct Page
pub const AND_A_IDX: u8 = 0x27;
/// Logical AND A Register Direct Page
pub const AND_A_D: u8 = 0x24;
/// Logical AND A Register Direct Page X Indexed
pub const AND_A_DX: u8 = 0x34;
/// Logical AND A Register Absolute
pub const AND_A_ABS: u8 = 0x25;
/// Logical AND A Register Absolute X Indexed
pub const AND_A_ABSX: u8 = 0x35;
/// Logical AND A Register Absolute Y Indexed
pub const AND_A_ABSY: u8 = 0x36;
/// Logical AND Direct Page Direct Page
pub const AND_D_D: u8 = 0x29;
/// Logical AND Direct Page Immediate
pub const AND_D_IMM: u8 = 0x38;
/// AND bit Carry Not Memory Bit
pub const AND1_C_NMB: u8 = 0x6A;
/// AND bit Carry Memory Bit
pub const AND1_C_MB: u8 = 0x4A;
/// Shift left A Register
pub const ASL_A: u8 = 0x1C;
/// Shift left Direct Page
pub const ASL_D: u8 = 0x0B;
/// Shift left Direct Page X Indexed
pub const ASL_DX: u8 = 0x1B;
/// Shift left Absolute
pub const ASL_ABS: u8 = 0x0C;
/// Branch if bit clear (Mask - the top 3 bits are set to 0)
/// Which bit is selected by the top 3 bits of the opcode
pub const BBC_D_R_MASK: u8 = 0x13;
/// Branch if bit set (Mask - the top 3 bits are set to 0)
/// Which bit is selected by the top 3 bits of the opcode
pub const BBS_D_R_MASK: u8 = 0x03;
/// Branch if carry clear Relative
pub const BCC_R: u8 = 0x90;
/// Branch if carry set Relative
pub const BCS_R: u8 = 0xB0;
/// Branch if equal Relative
pub const BEQ_R: u8 = 0xF0;
/// Branch if minus Relative
pub const BMI_R: u8 = 0x30;
/// Branch if not equal Relative
pub const BNE_R: u8 = 0xD0;
/// Branch if plus Relative
pub const BPL_R: u8 = 0x10;
/// Branch if overflow clear Relative
pub const BVC_R: u8 = 0x50;
/// Branch if overflow set Relative
pub const BVS_R: u8 = 0x70;
/// Branch always Relative
pub const BRA_R: u8 = 0x2F;
/// Break
pub const BRK: u8 = 0x0F;
/// Call subroutine Absolute
pub const CALL_ABS: u8 = 0x3F;
/// Compare and branch if not equal Direct Page X Indexed Relative
pub const CBNE_DX_R: u8 = 0xDE;
/// Compare and branch if not equal Direct Page Relative
pub const CBNE_D_R: u8 = 0x2E;
/// Clear bit 0nth Bit
pub const CLR1_D0: u8 = 0x12;
/// Clear bit 1nth Bit
pub const CLR1_D1: u8 = 0x32;
/// Clear bit 2nth Bit
pub const CLR1_D2: u8 = 0x52;
/// Clear bit 3nth Bit
pub const CLR1_D3: u8 = 0x72;
/// Clear bit 4nth Bit
pub const CLR1_D4: u8 = 0x92;
/// Clear bit 5nth Bit
pub const CLR1_D5: u8 = 0xB2;
/// Clear bit 6nth Bit
pub const CLR1_D6: u8 = 0xD2;
/// Clear bit 7nth Bit
pub const CLR1_D7: u8 = 0xF2;
/// Clear carry
pub const CLRC: u8 = 0x60;
/// Clear page flag
pub const CLRP: u8 = 0x20;
/// Clear overflow
pub const CLRV: u8 = 0xE0;
/// Compare values X Indexed Y Indexed
pub const CMP_IX_IY: u8 = 0x79;
/// Compare values A Register Immediate
pub const CMP_A_IMM: u8 = 0x68;
/// Compare values A Register X Indexed
pub const CMP_A_IX: u8 = 0x66;
/// Compare values A Register Y Indexed Indirect Direct Page
pub const CMP_A_IDY: u8 = 0x77;
/// Compare values A Register Indirect X Indexed Direct Page
pub const CMP_A_IDX: u8 = 0x67;
/// Compare values A Register Direct Page
pub const CMP_A_D: u8 = 0x64;
/// Compare values A Register Direct Page X Indexed
pub const CMP_A_DX: u8 = 0x74;
/// Compare values A Register Absolute
pub const CMP_A_ABS: u8 = 0x65;
/// Compare values A Register Absolute X Indexed
pub const CMP_A_ABSX: u8 = 0x75;
/// Compare values A Register Absolute Y Indexed
pub const CMP_A_ABSY: u8 = 0x76;
/// Compare values X Register Immediate
pub const CMP_X_IMM: u8 = 0xC8;
/// Compare values X Register Direct Page
pub const CMP_X_D: u8 = 0x3E;
/// Compare values X Register Absolute
pub const CMP_X_ABS: u8 = 0x1E;
/// Compare values Y Register Immediate
pub const CMP_Y_IMM: u8 = 0xAD;
/// Compare values Y Register Direct Page
pub const CMP_Y_D: u8 = 0x7E;
/// Compare values Y Register Absolute
pub const CMP_Y_ABS: u8 = 0x5E;
/// Compare values Direct Page Direct Page
pub const CMP_D_D: u8 = 0x69;
/// Compare values Direct Page Immediate
pub const CMP_D_IMM: u8 = 0x78;
/// Compare word Y Register and A Register Combined Direct Page
pub const CMPW_YA_D: u8 = 0x5A;
/// Decimal adjust (add) A Register
pub const DAA_A: u8 = 0xDF;
/// Decimal adjust (sub) A Register
pub const DAS_A: u8 = 0xBE;
/// Decrement and branch if not zero Y Register Relative
pub const DBNZ_Y_R: u8 = 0xFE;
/// Decrement and branch if not zero Direct Page Relative
pub const DBNZ_D_R: u8 = 0x6E;
/// Decrement A Register
pub const DEC_A: u8 = 0x9C;
/// Decrement X Register
pub const DEC_X: u8 = 0x1D;
/// Decrement Y Register
pub const DEC_Y: u8 = 0xDC;
/// Decrement Direct Page
pub const DEC_D: u8 = 0x8B;
/// Decrement Direct Page X Indexed
pub const DEC_DX: u8 = 0x9B;
/// Decrement Absolute
pub const DEC_ABS: u8 = 0x8C;
/// Decrement word Direct Page
pub const DECW_D: u8 = 0x1A;
/// Disable interrupts
pub const DI: u8 = 0xC0;
/// Divide Y Register and A Register Combined X Register
pub const DIV_YA_X: u8 = 0x9E;
/// Enable interrupts
pub const EI: u8 = 0xA0;
/// Exclusive OR X Indexed Y Indexed
pub const EOR_IX_IY: u8 = 0x59;
/// Exclusive OR A Register Immediate
pub const EOR_A_IMM: u8 = 0x48;
/// Exclusive OR A Register X Indexed
pub const EOR_A_IX: u8 = 0x46;
/// Exclusive OR A Register Y Indexed Indirect Direct Page
pub const EOR_A_IDY: u8 = 0x57;
/// Exclusive OR A Register Indirect X Indexed Direct Page
pub const EOR_A_IDX: u8 = 0x47;
/// Exclusive OR A Register Direct Page
pub const EOR_A_D: u8 = 0x44;
/// Exclusive OR A Register Direct Page X Indexed
pub const EOR_A_DX: u8 = 0x54;
/// Exclusive OR A Register Absolute
pub const EOR_A_ABS: u8 = 0x45;
/// Exclusive OR A Register Absolute X Indexed
pub const EOR_A_ABSX: u8 = 0x55;
/// Exclusive OR A Register Absolute Y Indexed
pub const EOR_A_ABSY: u8 = 0x56;
/// Exclusive OR Direct Page Direct Page
pub const EOR_D_D: u8 = 0x49;
/// Exclusive OR Direct Page Immediate
pub const EOR_D_IMM: u8 = 0x58;
/// EOR bit Carry Memory Bit
pub const EOR1_C_MB: u8 = 0x8A;
/// Increment A Register
pub const INC_A: u8 = 0xBC;
/// Increment X Register
pub const INC_X: u8 = 0x3D;
/// Increment Y Register
pub const INC_Y: u8 = 0xFC;
/// Increment Direct Page
pub const INC_D: u8 = 0xAB;
/// Increment Direct Page X Indexed
pub const INC_DX: u8 = 0xBB;
/// Increment Absolute
pub const INC_ABS: u8 = 0xAC;
/// Increment word Direct Page
pub const INCW_D: u8 = 0x3A;
/// Jump Indirect X Indexed Absolute
pub const JMP_IAX: u8 = 0x1F;
/// Jump Absolute
pub const JMP_ABS: u8 = 0x5F;
/// Shift right A Register
pub const LSR_A: u8 = 0x5C;
/// Shift right Direct Page
pub const LSR_D: u8 = 0x4B;
/// Shift right Direct Page X Indexed
pub const LSR_DX: u8 = 0x5B;
/// Shift right Absolute
pub const LSR_ABS: u8 = 0x4C;
/// Move data X Register with post increment A Register
pub const MOV_XINC_A: u8 = 0xAF;
/// Move data X Indexed A Register
pub const MOV_IX_A: u8 = 0xC6;
/// Move data Y Indexed Indirect Direct Page A Register
pub const MOV_IDY_A: u8 = 0xD7;
/// Move data Indirect X Indexed Direct Page A Register
pub const MOV_IDX_A: u8 = 0xC7;
/// Move data A Register Immediate
pub const MOV_A_IMM: u8 = 0xE8;
/// Move data A Register X Indexed
pub const MOV_A_IX: u8 = 0xE6;
/// Move data A Register X Register with post increment
pub const MOV_A_XINC: u8 = 0xBF;
/// Move data A Register Y Indexed Indirect Direct Page
pub const MOV_A_IDY: u8 = 0xF7;
/// Move data A Register Indirect X Indexed Direct Page
pub const MOV_A_IDX: u8 = 0xE7;
/// Move data A Register X Register
pub const MOV_A_X: u8 = 0x7D;
/// Move data A Register Y Register
pub const MOV_A_Y: u8 = 0xDD;
/// Move data A Register Direct Page
pub const MOV_A_D: u8 = 0xE4;
/// Move data A Register Direct Page X Indexed
pub const MOV_A_DX: u8 = 0xF4;
/// Move data A Register Absolute
pub const MOV_A_ABS: u8 = 0xE5;
/// Move data A Register Absolute X Indexed
pub const MOV_A_ABSX: u8 = 0xF5;
/// Move data A Register Absolute Y Indexed
pub const MOV_A_ABSY: u8 = 0xF6;
/// Move data Stack Pointer X Register
pub const MOV_SP_X: u8 = 0xBD;
/// Move data X Register Immediate
pub const MOV_X_IMM: u8 = 0xCD;
/// Move data X Register A Register
pub const MOV_X_A: u8 = 0x5D;
/// Move data X Register Stack Pointer
pub const MOV_X_SP: u8 = 0x9D;
/// Move data X Register Direct Page
pub const MOV_X_D: u8 = 0xF8;
/// Move data X Register Direct Page Y Indexed
pub const MOV_X_DY: u8 = 0xF9;
/// Move data X Register Absolute
pub const MOV_X_ABS: u8 = 0xE9;
/// Move data Y Register Immediate
pub const MOV_Y_IMM: u8 = 0x8D;
/// Move data Y Register A Register
pub const MOV_Y_A: u8 = 0xFD;
/// Move data Y Register Direct Page
pub const MOV_Y_D: u8 = 0xEB;
/// Move data Y Register Direct Page X Indexed
pub const MOV_Y_DX: u8 = 0xFB;
/// Move data Y Register Absolute
pub const MOV_Y_ABS: u8 = 0xEC;
/// Move data Direct Page Direct Page
pub const MOV_D_D: u8 = 0xFA;
/// Move data Direct Page X Indexed A Register
pub const MOV_DX_A: u8 = 0xD4;
/// Move data Direct Page X Indexed Y Register
pub const MOV_DX_Y: u8 = 0xDB;
/// Move data Direct Page Y Indexed X Register
pub const MOV_DY_X: u8 = 0xD9;
/// Move data Direct Page Immediate
pub const MOV_D_IMM: u8 = 0x8F;
/// Move data Direct Page A Register
pub const MOV_D_A: u8 = 0xC4;
/// Move data Direct Page X Register
pub const MOV_D_X: u8 = 0xD8;
/// Move data Direct Page Y Register
pub const MOV_D_Y: u8 = 0xCB;
/// Move data Absolute X Indexed A Register
pub const MOV_ABSX_A: u8 = 0xD5;
/// Move data Absolute Y Indexed A Register
pub const MOV_ABSY_A: u8 = 0xD6;
/// Move data Absolute A Register
pub const MOV_ABS_A: u8 = 0xC5;
/// Move data Absolute X Register
pub const MOV_ABS_X: u8 = 0xC9;
/// Move data Absolute Y Register
pub const MOV_ABS_Y: u8 = 0xCC;
/// Move bit Carry Memory Bit
pub const MOV1_C_MB: u8 = 0xAA;
/// Move bit Memory Bit Carry
pub const MOV1_MB_C: u8 = 0xCA;
/// Move word Y Register and A Register Combined Direct Page
pub const MOVW_YA_D: u8 = 0xBA;
/// Move word Direct Page Y Register and A Register Combined
pub const MOVW_D_YA: u8 = 0xDA;
/// Multiply Y Register and A Register Combined
pub const MUL_YA: u8 = 0xCF;
/// No operation
pub const NOP: u8 = 0x00;
/// Invert bit Memory Bit
pub const NOT1_MB: u8 = 0xEA;
/// Invert carry
pub const NOTC: u8 = 0xED;
/// Logical OR X Indexed Y Indexed
pub const OR_IX_IY: u8 = 0x19;
/// Logical OR A Register Immediate
pub const OR_A_IMM: u8 = 0x08;
/// Logical OR A Register X Indexed
pub const OR_A_IX: u8 = 0x06;
/// Logical OR A Register Y Indexed Indirect Direct Page
pub const OR_A_IDY: u8 = 0x17;
/// Logical OR A Register Indirect X Indexed Direct Page
pub const OR_A_IDX: u8 = 0x07;
/// Logical OR A Register Direct Page
pub const OR_A_D: u8 = 0x04;
/// Logical OR A Register Direct Page X Indexed
pub const OR_A_DX: u8 = 0x14;
/// Logical OR A Register Absolute
pub const OR_A_ABS: u8 = 0x05;
/// Logical OR A Register Absolute X Indexed
pub const OR_A_ABSX: u8 = 0x15;
/// Logical OR A Register Absolute Y Indexed
pub const OR_A_ABSY: u8 = 0x16;
/// Logical OR Direct Page Direct Page
pub const OR_D_D: u8 = 0x09;
/// Logical OR Direct Page Immediate
pub const OR_D_IMM: u8 = 0x18;
/// OR bit Carry Not Memory Bit
pub const OR1_C_NMB: u8 = 0x2A;
/// OR bit Carry Memory Bit
pub const OR1_C_MB: u8 = 0x0A;
/// Page call
pub const PCALL_: u8 = 0x4F;
/// Pop from stack A Register
pub const POP_A: u8 = 0xAE;
/// Pop from stack Processor Status Word
pub const POP_PSW: u8 = 0x8E;
/// Pop from stack X Register
pub const POP_X: u8 = 0xCE;
/// Pop from stack Y Register
pub const POP_Y: u8 = 0xEE;
/// Push to stack A Register
pub const PUSH_A: u8 = 0x2D;
/// Push to stack Processor Status Word
pub const PUSH_PSW: u8 = 0x0D;
/// Push to stack X Register
pub const PUSH_X: u8 = 0x4D;
/// Push to stack Y Register
pub const PUSH_Y: u8 = 0x6D;
/// Return
pub const RET: u8 = 0x6F;
/// Return from interrupt
pub const RETI: u8 = 0x7F;
/// Rotate left A Register
pub const ROL_A: u8 = 0x3C;
/// Rotate left Direct Page
pub const ROL_D: u8 = 0x2B;
/// Rotate left Direct Page X Indexed
pub const ROL_DX: u8 = 0x3B;
/// Rotate left Absolute
pub const ROL_ABS: u8 = 0x2C;
/// Rotate right A Register
pub const ROR_A: u8 = 0x7C;
/// Rotate right Direct Page
pub const ROR_D: u8 = 0x6B;
/// Rotate right Direct Page X Indexed
pub const ROR_DX: u8 = 0x7B;
/// Rotate right Absolute
pub const ROR_ABS: u8 = 0x6C;
/// Subtract with carry X Indexed Y Indexed
pub const SBC_IX_IY: u8 = 0xB9;
/// Subtract with carry A Register Immediate
pub const SBC_A_IMM: u8 = 0xA8;
/// Subtract with carry A Register X Indexed
pub const SBC_A_IX: u8 = 0xA6;
/// Subtract with carry A Register Y Indexed Indirect Direct Page
pub const SBC_A_IDY: u8 = 0xB7;
/// Subtract with carry A Register Indirect X Indexed Direct Page
pub const SBC_A_IDX: u8 = 0xA7;
/// Subtract with carry A Register Direct Page
pub const SBC_A_D: u8 = 0xA4;
/// Subtract with carry A Register Direct Page X Indexed
pub const SBC_A_DX: u8 = 0xB4;
/// Subtract with carry A Register Absolute
pub const SBC_A_ABS: u8 = 0xA5;
/// Subtract with carry A Register Absolute X Indexed
pub const SBC_A_ABSX: u8 = 0xB5;
/// Subtract with carry A Register Absolute Y Indexed
pub const SBC_A_ABSY: u8 = 0xB6;
/// Subtract with carry Direct Page Direct Page
pub const SBC_D_D: u8 = 0xA9;
/// Subtract with carry Direct Page Immediate
pub const SBC_D_IMM: u8 = 0xB8;
/// Set bit 0nth Bit
pub const SET1_D0: u8 = 0x02;
/// Set bit 1nth Bit
pub const SET1_D1: u8 = 0x22;
/// Set bit 2nth Bit
pub const SET1_D2: u8 = 0x42;
/// Set bit 3nth Bit
pub const SET1_D3: u8 = 0x62;
/// Set bit 4nth Bit
pub const SET1_D4: u8 = 0x82;
/// Set bit 5nth Bit
pub const SET1_D5: u8 = 0xA2;
/// Set bit 6nth Bit
pub const SET1_D6: u8 = 0xC2;
/// Set bit 7nth Bit
pub const SET1_D7: u8 = 0xE2;
/// Set carry
pub const SETC: u8 = 0x80;
/// Set page flag
pub const SETP: u8 = 0x40;
/// Sleep mode
pub const SLEEP: u8 = 0xEF;
/// Stop processor
pub const STOP: u8 = 0xFF;
/// Subtract word Y Register and A Register Combined Direct Page
pub const SUBW_YA_D: u8 = 0x9A;
/// Table call 0xFFDE + 0
pub const TCALL_0: u8 = 0x01;
/// Table call 0xFFDE + 1
pub const TCALL_1: u8 = 0x11;
/// Table call 0xFFDE + 2
pub const TCALL_2: u8 = 0x21;
/// Table call 0xFFDE + 3
pub const TCALL_3: u8 = 0x31;
/// Table call 0xFFDE + 4
pub const TCALL_4: u8 = 0x41;
/// Table call 0xFFDE + 5
pub const TCALL_5: u8 = 0x51;
/// Table call 0xFFDE + 6
pub const TCALL_6: u8 = 0x61;
/// Table call 0xFFDE + 7
pub const TCALL_7: u8 = 0x71;
/// Table call 0xFFDE + 8
pub const TCALL_8: u8 = 0x81;
/// Table call 0xFFDE + 9
pub const TCALL_9: u8 = 0x91;
/// Table call 0xFFDE + 10
pub const TCALL_10: u8 = 0xA1;
/// Table call 0xFFDE + 11
pub const TCALL_11: u8 = 0xB1;
/// Table call 0xFFDE + 12
pub const TCALL_12: u8 = 0xC1;
/// Table call 0xFFDE + 13
pub const TCALL_13: u8 = 0xD1;
/// Table call 0xFFDE + 14
pub const TCALL_14: u8 = 0xE1;
/// Table call 0xFFDE + 15
pub const TCALL_15: u8 = 0xF1;
/// Test and clear bit Absolute
pub const TCLR1_ABS: u8 = 0x4E;
/// Test and set bit Absolute
pub const TSET1_ABS: u8 = 0x0E;
/// Swap nibbles A Register
pub const XCN_A: u8 = 0x9F;
