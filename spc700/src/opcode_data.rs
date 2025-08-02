use crate::opcodes::*;

pub enum AddressMode {
    A,
    X,
    Y,
    Imm,
    D,
    Dx,
    Dy,
    Ix,
    Iy,
    Idx,
    Idy,
    Abs,
    AbsX,
    AbsY,
    Rel,
    Mb,
    Nmb,
    C,
    Sp,
    Psw,
}
pub struct OpcodeData {
    pub name: &'static str,
    pub addr_modes: Vec<AddressMode>,
}

impl OpcodeData {
    pub fn from_opcode(opcode: u8) -> OpcodeData {
        match opcode {
            ADC_IX_IY => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            ADC_A_IMM => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            CALL_ABS => OpcodeData {
                name: "CALL",
                addr_modes: vec![AddressMode::Imm],
            },
            ADC_A_IX => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            ADC_A_IDY => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            ADC_A_IDX => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            ADC_A_D => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            ADC_A_DX => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            ADC_A_ABS => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            ADC_A_ABSX => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            ADC_A_ABSY => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            ADC_D_D => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            ADC_D_IMM => OpcodeData {
                name: "ADC",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            ADDW_YA_D => OpcodeData {
                name: "ADDW",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            AND_IX_IY => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            AND_A_IMM => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            AND_A_IX => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            AND_A_IDY => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            AND_A_IDX => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            AND_A_D => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            AND_A_DX => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            AND_A_ABS => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            AND_A_ABSX => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            AND_A_ABSY => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            AND_D_D => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            AND_D_IMM => OpcodeData {
                name: "AND",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            AND1_C_NMB => OpcodeData {
                name: "AND1",
                addr_modes: vec![AddressMode::Nmb],
            },
            AND1_C_MB => OpcodeData {
                name: "AND1",
                addr_modes: vec![AddressMode::Mb],
            },
            ASL_A => OpcodeData {
                name: "ASL",
                addr_modes: vec![AddressMode::A],
            },
            ASL_D => OpcodeData {
                name: "ASL",
                addr_modes: vec![AddressMode::D],
            },
            ASL_DX => OpcodeData {
                name: "ASL",
                addr_modes: vec![AddressMode::Dx],
            },
            ASL_ABS => OpcodeData {
                name: "ASL",
                addr_modes: vec![AddressMode::Abs],
            },
            // BBS
            o if opcode & 0x1F == BBS_D_R_MASK => todo!(),
            // BBC
            o if opcode & 0x1F == BBC_D_R_MASK => todo!(),
            BCC_R => OpcodeData {
                name: "BCC",
                addr_modes: vec![AddressMode::Rel],
            },
            BCS_R => OpcodeData {
                name: "BCS",
                addr_modes: vec![AddressMode::Rel],
            },
            BEQ_R => OpcodeData {
                name: "BEQ",
                addr_modes: vec![AddressMode::Rel],
            },
            BMI_R => OpcodeData {
                name: "BMI",
                addr_modes: vec![AddressMode::Rel],
            },
            BNE_R => OpcodeData {
                name: "BNE",
                addr_modes: vec![AddressMode::Rel],
            },
            BPL_R => OpcodeData {
                name: "BPL",
                addr_modes: vec![AddressMode::Rel],
            },
            BVC_R => OpcodeData {
                name: "BVC",
                addr_modes: vec![AddressMode::Rel],
            },
            BVS_R => OpcodeData {
                name: "BVS",
                addr_modes: vec![AddressMode::Rel],
            },
            BRA_R => OpcodeData {
                name: "BRA",
                addr_modes: vec![AddressMode::Rel],
            },
            BRK => OpcodeData {
                name: "BRK",
                addr_modes: vec![],
            },
            CBNE_DX_R => OpcodeData {
                name: "CBNE",
                addr_modes: vec![AddressMode::Dx, AddressMode::Rel],
            },
            CBNE_D_R => OpcodeData {
                name: "CBNE",
                addr_modes: vec![AddressMode::D, AddressMode::Rel],
            },
            o if opcode & 0x1F == CLR1_D => todo!(),
            CLRC => OpcodeData {
                name: "CLRC",
                addr_modes: vec![],
            },
            CLRP => OpcodeData {
                name: "CLRP",
                addr_modes: vec![],
            },
            CLRV => OpcodeData {
                name: "CLRV",
                addr_modes: vec![],
            },
            CMP_IX_IY => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            CMP_A_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            CMP_A_IX => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            CMP_A_IDY => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            CMP_A_IDX => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            CMP_A_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            CMP_A_DX => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            CMP_A_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            CMP_A_ABSX => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            CMP_A_ABSY => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            CMP_X_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::X, AddressMode::Imm],
            },
            CMP_X_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::X, AddressMode::D],
            },
            CMP_X_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::X, AddressMode::Abs],
            },
            CMP_Y_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::Y, AddressMode::Imm],
            },
            CMP_Y_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::Y, AddressMode::D],
            },
            CMP_Y_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::Y, AddressMode::Abs],
            },
            CMP_D_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            CMP_D_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            CMPW_YA_D => OpcodeData {
                name: "CMPW",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            DAA_A => OpcodeData {
                name: "DAA",
                addr_modes: vec![AddressMode::A],
            },
            DAS_A => OpcodeData {
                name: "DAS",
                addr_modes: vec![AddressMode::A],
            },
            DBNZ_Y_R => OpcodeData {
                name: "DBNZ",
                addr_modes: vec![AddressMode::Y, AddressMode::Rel],
            },
            DBNZ_D_R => OpcodeData {
                name: "DBNZ",
                addr_modes: vec![AddressMode::D, AddressMode::Rel],
            },
            DEC_A => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::A],
            },
            DEC_X => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::X],
            },
            DEC_Y => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::Y],
            },
            DEC_D => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::D],
            },
            DEC_DX => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::Dx],
            },
            DEC_ABS => OpcodeData {
                name: "DEC",
                addr_modes: vec![AddressMode::Abs],
            },
            DECW_D => OpcodeData {
                name: "DECW",
                addr_modes: vec![AddressMode::D],
            },
            DI => OpcodeData {
                name: "DI",
                addr_modes: vec![],
            },
            DIV_YA_X => OpcodeData {
                name: "DIV",
                addr_modes: vec![AddressMode::A, AddressMode::X],
            },
            EI => OpcodeData {
                name: "EI",
                addr_modes: vec![],
            },
            EOR_IX_IY => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            EOR_A_IMM => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            EOR_A_IX => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            EOR_A_IDY => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            EOR_A_IDX => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            EOR_A_D => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            EOR_A_DX => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            EOR_A_ABS => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            EOR_A_ABSX => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            EOR_A_ABSY => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            EOR_D_D => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            EOR_D_IMM => OpcodeData {
                name: "EOR",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            EOR1_C_MB => OpcodeData {
                name: "EOR1",
                addr_modes: vec![AddressMode::Mb],
            },
            INC_A => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::A],
            },
            INC_X => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::X],
            },
            INC_Y => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::Y],
            },
            INC_D => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::D],
            },
            INC_DX => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::Dx],
            },
            INC_ABS => OpcodeData {
                name: "INC",
                addr_modes: vec![AddressMode::Abs],
            },
            INCW_D => OpcodeData {
                name: "INCW",
                addr_modes: vec![AddressMode::D],
            },
            JMP_IAX => OpcodeData {
                name: "JMP",
                addr_modes: vec![AddressMode::Abs, AddressMode::Ix],
            },
            JMP_ABS => OpcodeData {
                name: "JMP",
                addr_modes: vec![AddressMode::Abs],
            },
            LSR_A => OpcodeData {
                name: "LSR",
                addr_modes: vec![AddressMode::A],
            },
            LSR_D => OpcodeData {
                name: "LSR",
                addr_modes: vec![AddressMode::D],
            },
            LSR_DX => OpcodeData {
                name: "LSR",
                addr_modes: vec![AddressMode::Dx],
            },
            LSR_ABS => OpcodeData {
                name: "LSR",
                addr_modes: vec![AddressMode::Abs],
            },
            MOV_XINC_A => todo!(),
            MOV_IX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Ix, AddressMode::A],
            },
            MOV_IDY_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Iy, AddressMode::A],
            },
            MOV_IDX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Idx, AddressMode::A],
            },
            MOV_A_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            MOV_A_IX => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            MOV_A_XINC => todo!(),
            MOV_A_IDY => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            MOV_A_IDX => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Idx],
            },
            MOV_A_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::X],
            },
            MOV_A_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Y],
            },
            MOV_A_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            MOV_A_DX => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            MOV_A_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            MOV_A_ABSX => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            MOV_A_ABSY => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            MOV_SP_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Sp, AddressMode::X],
            },
            MOV_X_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::Imm],
            },
            MOV_X_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::A],
            },
            MOV_X_SP => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::Sp],
            },
            MOV_X_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::D],
            },
            MOV_X_DY => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::Dy],
            },
            MOV_X_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::X, AddressMode::Abs],
            },
            MOV_Y_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Y, AddressMode::Imm],
            },
            MOV_Y_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Y, AddressMode::A],
            },
            MOV_Y_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Y, AddressMode::D],
            },
            MOV_Y_DX => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Y, AddressMode::Dx],
            },
            MOV_Y_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Y, AddressMode::Abs],
            },
            MOV_D_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            MOV_DX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Dx, AddressMode::A],
            },
            MOV_DX_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Dx, AddressMode::Y],
            },
            MOV_DY_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Dy, AddressMode::X],
            },
            MOV_D_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            MOV_D_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::D, AddressMode::A],
            },
            MOV_D_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::D, AddressMode::X],
            },
            MOV_D_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::D, AddressMode::Y],
            },
            MOV_ABSX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::AbsX, AddressMode::A],
            },
            MOV_ABSY_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::AbsY, AddressMode::A],
            },
            MOV_ABS_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Abs, AddressMode::A],
            },
            MOV_ABS_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Abs, AddressMode::X],
            },
            MOV_ABS_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![AddressMode::Abs, AddressMode::Y],
            },
            MOV1_C_MB => OpcodeData {
                name: "MOV1",
                addr_modes: vec![AddressMode::C, AddressMode::Mb],
            },
            MOV1_MB_C => OpcodeData {
                name: "MOV1",
                addr_modes: vec![AddressMode::Mb, AddressMode::C],
            },
            MOVW_YA_D => OpcodeData {
                name: "MOVW",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            MOVW_D_YA => OpcodeData {
                name: "MOVW",
                addr_modes: vec![AddressMode::D, AddressMode::A],
            },
            MUL_YA => OpcodeData {
                name: "MUL",
                addr_modes: vec![AddressMode::A],
            },
            NOP => OpcodeData {
                name: "NOP",
                addr_modes: vec![],
            },
            NOT1_MB => OpcodeData {
                name: "NOT1",
                addr_modes: vec![AddressMode::Mb],
            },
            NOTC => OpcodeData {
                name: "NOTC",
                addr_modes: vec![],
            },
            OR_IX_IY => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            OR_A_IMM => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            OR_A_IX => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            OR_A_IDY => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            OR_A_IDX => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            OR_A_D => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            OR_A_DX => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            OR_A_ABS => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            OR_A_ABSX => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            OR_A_ABSY => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            OR_D_D => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            OR_D_IMM => OpcodeData {
                name: "OR",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            OR1_C_NMB => OpcodeData {
                name: "OR1",
                addr_modes: vec![AddressMode::C, AddressMode::Nmb],
            },
            OR1_C_MB => OpcodeData {
                name: "OR1",
                addr_modes: vec![AddressMode::C, AddressMode::Mb],
            },
            PCALL => OpcodeData {
                name: "PCALL",
                addr_modes: vec![AddressMode::Imm],
            },
            POP_A => OpcodeData {
                name: "POP",
                addr_modes: vec![AddressMode::A],
            },
            POP_PSW => OpcodeData {
                name: "POP",
                addr_modes: vec![AddressMode::Psw],
            },
            POP_X => OpcodeData {
                name: "POP",
                addr_modes: vec![AddressMode::X],
            },
            POP_Y => OpcodeData {
                name: "POP",
                addr_modes: vec![AddressMode::Y],
            },
            PUSH_A => OpcodeData {
                name: "PUSH",
                addr_modes: vec![AddressMode::A],
            },
            PUSH_PSW => OpcodeData {
                name: "PUSH",
                addr_modes: vec![AddressMode::Psw],
            },
            PUSH_X => OpcodeData {
                name: "PUSH",
                addr_modes: vec![AddressMode::X],
            },
            PUSH_Y => OpcodeData {
                name: "PUSH",
                addr_modes: vec![AddressMode::Y],
            },
            RET => OpcodeData {
                name: "RET",
                addr_modes: vec![],
            },
            RETI => OpcodeData {
                name: "RETI",
                addr_modes: vec![],
            },
            ROL_A => OpcodeData {
                name: "ROL",
                addr_modes: vec![AddressMode::A],
            },
            ROL_D => OpcodeData {
                name: "ROL",
                addr_modes: vec![AddressMode::D],
            },
            ROL_DX => OpcodeData {
                name: "ROL",
                addr_modes: vec![AddressMode::Dx],
            },
            ROL_ABS => OpcodeData {
                name: "ROL",
                addr_modes: vec![AddressMode::Abs],
            },
            ROR_A => OpcodeData {
                name: "ROR",
                addr_modes: vec![AddressMode::A],
            },
            ROR_D => OpcodeData {
                name: "ROR",
                addr_modes: vec![AddressMode::D],
            },
            ROR_DX => OpcodeData {
                name: "ROR",
                addr_modes: vec![AddressMode::Dx],
            },
            ROR_ABS => OpcodeData {
                name: "ROR",
                addr_modes: vec![AddressMode::Abs],
            },
            SBC_IX_IY => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::Ix, AddressMode::Iy],
            },
            SBC_A_IMM => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Imm],
            },
            SBC_A_IX => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            SBC_A_IDY => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Iy],
            },
            SBC_A_IDX => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Ix],
            },
            SBC_A_D => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            SBC_A_DX => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Dx],
            },
            SBC_A_ABS => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::Abs],
            },
            SBC_A_ABSX => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::AbsX],
            },
            SBC_A_ABSY => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::A, AddressMode::AbsY],
            },
            SBC_D_D => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::D, AddressMode::D],
            },
            SBC_D_IMM => OpcodeData {
                name: "SBC",
                addr_modes: vec![AddressMode::D, AddressMode::Imm],
            },
            opcode if opcode & 0x1F == SET1_MASK => todo!(),
            SETC => OpcodeData {
                name: "SETC",
                addr_modes: vec![],
            },
            SETP => OpcodeData {
                name: "SETP",
                addr_modes: vec![],
            },
            SLEEP => OpcodeData {
                name: "SLEEP",
                addr_modes: vec![],
            },
            STOP => OpcodeData {
                name: "STOP",
                addr_modes: vec![],
            },
            SUBW_YA_D => OpcodeData {
                name: "SUBW",
                addr_modes: vec![AddressMode::A, AddressMode::D],
            },
            o if opcode & 0x0F == TCALL_MASK => todo!(),
            TCLR1_ABS => OpcodeData {
                name: "TCLR1",
                addr_modes: vec![AddressMode::Abs],
            },
            TSET1_ABS => OpcodeData {
                name: "TSET1",
                addr_modes: vec![AddressMode::Abs],
            },
            XCN_A => OpcodeData {
                name: "XCN",
                addr_modes: vec![AddressMode::A],
            },
            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }
}
