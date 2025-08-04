use crate::opcodes::*;

pub enum AddressMode {
    A,
    X,
    XInc,
    Y,
    Ya,
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
    Iax,
    Rel,
    Mb,
    Nmb,
    C,
    Sp,
    Psw,
    // Number encoded in the opcode
    Encoded(u8),
}
pub struct OpcodeData {
    pub name: &'static str,
    pub addr_modes: Vec<AddressMode>,
}

pub fn format_address_modes(modes: &[AddressMode], values: &[u8]) -> String {
    let mut i = 0;
    let mut v = move || {
        let v = values[i];
        i += 1;
        v
    };
    // Need to reverse since the operands for the second address mode comes first
    let mut m = modes
        .iter()
        .rev()
        .map(move |mode| match mode {
            AddressMode::A => "A".to_string(),
            AddressMode::X => "X".to_string(),
            AddressMode::XInc => "XINC".to_string(),
            AddressMode::Y => "Y".to_string(),
            AddressMode::Ya => "YA".to_string(),
            AddressMode::Imm => format!("#${:02X}", v()),
            AddressMode::Iax => format!("[${:04X}]", u16::from_le_bytes([v(), v()])),
            AddressMode::D => format!("${:02X}", v()),
            AddressMode::Dx => format!("${:02X}, X", v()),
            AddressMode::Dy => format!("${:02X}, Y", v()),
            AddressMode::Ix => "(X)".to_string(),
            AddressMode::Iy => "(Y)".to_string(),
            AddressMode::Idx => format!("([${:02X} + X])", v()),
            AddressMode::Idy => format!("([${:02X}] + Y)", v()),
            AddressMode::Abs => format!("${:04X}", u16::from_le_bytes([v(), v()])),
            AddressMode::AbsX => format!("${:04X}, X", u16::from_le_bytes([v(), v()])),
            AddressMode::AbsY => format!("${:04X}, Y", u16::from_le_bytes([v(), v()])),
            AddressMode::Rel => format!("{:+}", v() as i8),
            AddressMode::Mb => "Mb".to_string(),
            AddressMode::Nmb => "Nmb".to_string(),
            AddressMode::C => "C".to_string(),
            AddressMode::Sp => "SP".to_string(),
            AddressMode::Psw => "PSW".to_string(),
            AddressMode::Encoded(n) => format!("bit{:X}", n),
        })
        .collect::<Vec<_>>();
    // Undo the reversing before formatting
    m.reverse();
    m.join(", ")
}

impl OpcodeData {
    pub fn from_opcode(opcode: u8) -> OpcodeData {
        use AddressMode::*;
        match opcode {
            ADC_IX_IY => OpcodeData {
                name: "ADC",
                addr_modes: vec![Ix, Iy],
            },
            ADC_A_IMM => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Imm],
            },
            CALL_ABS => OpcodeData {
                name: "CALL",
                addr_modes: vec![Abs],
            },
            ADC_A_IX => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Ix],
            },
            ADC_A_IDY => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Idy],
            },
            ADC_A_IDX => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Idx],
            },
            ADC_A_D => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, D],
            },
            ADC_A_DX => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Dx],
            },
            ADC_A_ABS => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, Abs],
            },
            ADC_A_ABSX => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, AbsX],
            },
            ADC_A_ABSY => OpcodeData {
                name: "ADC",
                addr_modes: vec![A, AbsY],
            },
            ADC_D_D => OpcodeData {
                name: "ADC",
                addr_modes: vec![D, D],
            },
            ADC_D_IMM => OpcodeData {
                name: "ADC",
                addr_modes: vec![D, Imm],
            },
            ADDW_YA_D => OpcodeData {
                name: "ADDW",
                addr_modes: vec![Ya, D],
            },
            AND_IX_IY => OpcodeData {
                name: "AND",
                addr_modes: vec![Ix, Iy],
            },
            AND_A_IMM => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Imm],
            },
            AND_A_IX => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Ix],
            },
            AND_A_IDY => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Idy],
            },
            AND_A_IDX => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Idx],
            },
            AND_A_D => OpcodeData {
                name: "AND",
                addr_modes: vec![A, D],
            },
            AND_A_DX => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Dx],
            },
            AND_A_ABS => OpcodeData {
                name: "AND",
                addr_modes: vec![A, Abs],
            },
            AND_A_ABSX => OpcodeData {
                name: "AND",
                addr_modes: vec![A, AbsX],
            },
            AND_A_ABSY => OpcodeData {
                name: "AND",
                addr_modes: vec![A, AbsY],
            },
            AND_D_D => OpcodeData {
                name: "AND",
                addr_modes: vec![D, D],
            },
            AND_D_IMM => OpcodeData {
                name: "AND",
                addr_modes: vec![D, Imm],
            },
            AND1_C_NMB => OpcodeData {
                name: "AND1",
                addr_modes: vec![Nmb],
            },
            AND1_C_MB => OpcodeData {
                name: "AND1",
                addr_modes: vec![Mb],
            },
            ASL_A => OpcodeData {
                name: "ASL",
                addr_modes: vec![A],
            },
            ASL_D => OpcodeData {
                name: "ASL",
                addr_modes: vec![D],
            },
            ASL_DX => OpcodeData {
                name: "ASL",
                addr_modes: vec![Dx],
            },
            ASL_ABS => OpcodeData {
                name: "ASL",
                addr_modes: vec![Abs],
            },
            // BBS
            _ if opcode & 0x1F == BBS_D_R_MASK => OpcodeData {
                name: "BBS",
                addr_modes: vec![Encoded(opcode >> 5), D, Rel],
            },
            // BBC
            _ if opcode & 0x1F == BBC_D_R_MASK => OpcodeData {
                name: "BBC",
                addr_modes: vec![Encoded(opcode >> 5), D, Rel],
            },
            BCC_R => OpcodeData {
                name: "BCC",
                addr_modes: vec![Rel],
            },
            BCS_R => OpcodeData {
                name: "BCS",
                addr_modes: vec![Rel],
            },
            BEQ_R => OpcodeData {
                name: "BEQ",
                addr_modes: vec![Rel],
            },
            BMI_R => OpcodeData {
                name: "BMI",
                addr_modes: vec![Rel],
            },
            BNE_R => OpcodeData {
                name: "BNE",
                addr_modes: vec![Rel],
            },
            BPL_R => OpcodeData {
                name: "BPL",
                addr_modes: vec![Rel],
            },
            BVC_R => OpcodeData {
                name: "BVC",
                addr_modes: vec![Rel],
            },
            BVS_R => OpcodeData {
                name: "BVS",
                addr_modes: vec![Rel],
            },
            BRA_R => OpcodeData {
                name: "BRA",
                addr_modes: vec![Rel],
            },
            BRK => OpcodeData {
                name: "BRK",
                addr_modes: vec![],
            },
            CBNE_DX_R => OpcodeData {
                name: "CBNE",
                addr_modes: vec![Dx, Rel],
            },
            CBNE_D_R => OpcodeData {
                name: "CBNE",
                addr_modes: vec![D, Rel],
            },
            _ if opcode & 0x1F == CLR1_D => OpcodeData {
                name: "CLR1",
                addr_modes: vec![Encoded(opcode >> 5), D],
            },
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
                addr_modes: vec![Ix, Iy],
            },
            CMP_A_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Imm],
            },
            CMP_A_IX => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Ix],
            },
            CMP_A_IDY => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Idy],
            },
            CMP_A_IDX => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Idx],
            },
            CMP_A_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, D],
            },
            CMP_A_DX => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Dx],
            },
            CMP_A_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, Abs],
            },
            CMP_A_ABSX => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, AbsX],
            },
            CMP_A_ABSY => OpcodeData {
                name: "CMP",
                addr_modes: vec![A, AbsY],
            },
            CMP_X_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![X, Imm],
            },
            CMP_X_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![X, D],
            },
            CMP_X_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![X, Abs],
            },
            CMP_Y_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![Y, Imm],
            },
            CMP_Y_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![Y, D],
            },
            CMP_Y_ABS => OpcodeData {
                name: "CMP",
                addr_modes: vec![Y, Abs],
            },
            CMP_D_D => OpcodeData {
                name: "CMP",
                addr_modes: vec![D, D],
            },
            CMP_D_IMM => OpcodeData {
                name: "CMP",
                addr_modes: vec![D, Imm],
            },
            CMPW_YA_D => OpcodeData {
                name: "CMPW",
                addr_modes: vec![Ya, D],
            },
            DAA_A => OpcodeData {
                name: "DAA",
                addr_modes: vec![A],
            },
            DAS_A => OpcodeData {
                name: "DAS",
                addr_modes: vec![A],
            },
            DBNZ_Y_R => OpcodeData {
                name: "DBNZ",
                addr_modes: vec![Y, Rel],
            },
            DBNZ_D_R => OpcodeData {
                name: "DBNZ",
                addr_modes: vec![D, Rel],
            },
            DEC_A => OpcodeData {
                name: "DEC",
                addr_modes: vec![A],
            },
            DEC_X => OpcodeData {
                name: "DEC",
                addr_modes: vec![X],
            },
            DEC_Y => OpcodeData {
                name: "DEC",
                addr_modes: vec![Y],
            },
            DEC_D => OpcodeData {
                name: "DEC",
                addr_modes: vec![D],
            },
            DEC_DX => OpcodeData {
                name: "DEC",
                addr_modes: vec![Dx],
            },
            DEC_ABS => OpcodeData {
                name: "DEC",
                addr_modes: vec![Abs],
            },
            DECW_D => OpcodeData {
                name: "DECW",
                addr_modes: vec![D],
            },
            DI => OpcodeData {
                name: "DI",
                addr_modes: vec![],
            },
            DIV_YA_X => OpcodeData {
                name: "DIV",
                addr_modes: vec![Ya, X],
            },
            EI => OpcodeData {
                name: "EI",
                addr_modes: vec![],
            },
            EOR_IX_IY => OpcodeData {
                name: "EOR",
                addr_modes: vec![Ix, Iy],
            },
            EOR_A_IMM => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Imm],
            },
            EOR_A_IX => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Ix],
            },
            EOR_A_IDY => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Idy],
            },
            EOR_A_IDX => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Idx],
            },
            EOR_A_D => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, D],
            },
            EOR_A_DX => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Dx],
            },
            EOR_A_ABS => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, Abs],
            },
            EOR_A_ABSX => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, AbsX],
            },
            EOR_A_ABSY => OpcodeData {
                name: "EOR",
                addr_modes: vec![A, AbsY],
            },
            EOR_D_D => OpcodeData {
                name: "EOR",
                addr_modes: vec![D, D],
            },
            EOR_D_IMM => OpcodeData {
                name: "EOR",
                addr_modes: vec![D, Imm],
            },
            EOR1_C_MB => OpcodeData {
                name: "EOR1",
                addr_modes: vec![Mb],
            },
            INC_A => OpcodeData {
                name: "INC",
                addr_modes: vec![A],
            },
            INC_X => OpcodeData {
                name: "INC",
                addr_modes: vec![X],
            },
            INC_Y => OpcodeData {
                name: "INC",
                addr_modes: vec![Y],
            },
            INC_D => OpcodeData {
                name: "INC",
                addr_modes: vec![D],
            },
            INC_DX => OpcodeData {
                name: "INC",
                addr_modes: vec![Dx],
            },
            INC_ABS => OpcodeData {
                name: "INC",
                addr_modes: vec![Abs],
            },
            INCW_D => OpcodeData {
                name: "INCW",
                addr_modes: vec![D],
            },
            JMP_IAX => OpcodeData {
                name: "JMP",
                addr_modes: vec![Iax],
            },
            JMP_ABS => OpcodeData {
                name: "JMP",
                addr_modes: vec![Abs],
            },
            LSR_A => OpcodeData {
                name: "LSR",
                addr_modes: vec![A],
            },
            LSR_D => OpcodeData {
                name: "LSR",
                addr_modes: vec![D],
            },
            LSR_DX => OpcodeData {
                name: "LSR",
                addr_modes: vec![Dx],
            },
            LSR_ABS => OpcodeData {
                name: "LSR",
                addr_modes: vec![Abs],
            },
            MOV_XINC_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![XInc, A],
            },
            MOV_IX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Ix, A],
            },
            MOV_IDY_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Idy, A],
            },
            MOV_IDX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Idx, A],
            },
            MOV_A_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Imm],
            },
            MOV_A_IX => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Ix],
            },
            MOV_A_XINC => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, XInc],
            },
            MOV_A_IDY => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Idy],
            },
            MOV_A_IDX => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Idx],
            },
            MOV_A_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, X],
            },
            MOV_A_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Y],
            },
            MOV_A_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, D],
            },
            MOV_A_DX => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Dx],
            },
            MOV_A_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, Abs],
            },
            MOV_A_ABSX => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, AbsX],
            },
            MOV_A_ABSY => OpcodeData {
                name: "MOV",
                addr_modes: vec![A, AbsY],
            },
            MOV_SP_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![Sp, X],
            },
            MOV_X_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, Imm],
            },
            MOV_X_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, A],
            },
            MOV_X_SP => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, Sp],
            },
            MOV_X_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, D],
            },
            MOV_X_DY => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, Dy],
            },
            MOV_X_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![X, Abs],
            },
            MOV_Y_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![Y, Imm],
            },
            MOV_Y_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Y, A],
            },
            MOV_Y_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![Y, D],
            },
            MOV_Y_DX => OpcodeData {
                name: "MOV",
                addr_modes: vec![Y, Dx],
            },
            MOV_Y_ABS => OpcodeData {
                name: "MOV",
                addr_modes: vec![Y, Abs],
            },
            MOV_D_D => OpcodeData {
                name: "MOV",
                addr_modes: vec![D, D],
            },
            MOV_DX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Dx, A],
            },
            MOV_DX_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![Dx, Y],
            },
            MOV_DY_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![Dy, X],
            },
            MOV_D_IMM => OpcodeData {
                name: "MOV",
                addr_modes: vec![D, Imm],
            },
            MOV_D_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![D, A],
            },
            MOV_D_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![D, X],
            },
            MOV_D_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![D, Y],
            },
            MOV_ABSX_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AbsX, A],
            },
            MOV_ABSY_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![AbsY, A],
            },
            MOV_ABS_A => OpcodeData {
                name: "MOV",
                addr_modes: vec![Abs, A],
            },
            MOV_ABS_X => OpcodeData {
                name: "MOV",
                addr_modes: vec![Abs, X],
            },
            MOV_ABS_Y => OpcodeData {
                name: "MOV",
                addr_modes: vec![Abs, Y],
            },
            MOV1_C_MB => OpcodeData {
                name: "MOV1",
                addr_modes: vec![C, Mb],
            },
            MOV1_MB_C => OpcodeData {
                name: "MOV1",
                addr_modes: vec![Mb, C],
            },
            MOVW_YA_D => OpcodeData {
                name: "MOVW",
                addr_modes: vec![Ya, D],
            },
            MOVW_D_YA => OpcodeData {
                name: "MOVW",
                addr_modes: vec![D, Ya],
            },
            MUL_YA => OpcodeData {
                name: "MUL",
                addr_modes: vec![Ya],
            },
            NOP => OpcodeData {
                name: "NOP",
                addr_modes: vec![],
            },
            NOT1_MB => OpcodeData {
                name: "NOT1",
                addr_modes: vec![Mb],
            },
            NOTC => OpcodeData {
                name: "NOTC",
                addr_modes: vec![],
            },
            OR_IX_IY => OpcodeData {
                name: "OR",
                addr_modes: vec![Ix, Iy],
            },
            OR_A_IMM => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Imm],
            },
            OR_A_IX => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Ix],
            },
            OR_A_IDY => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Idy],
            },
            OR_A_IDX => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Idx],
            },
            OR_A_D => OpcodeData {
                name: "OR",
                addr_modes: vec![A, D],
            },
            OR_A_DX => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Dx],
            },
            OR_A_ABS => OpcodeData {
                name: "OR",
                addr_modes: vec![A, Abs],
            },
            OR_A_ABSX => OpcodeData {
                name: "OR",
                addr_modes: vec![A, AbsX],
            },
            OR_A_ABSY => OpcodeData {
                name: "OR",
                addr_modes: vec![A, AbsY],
            },
            OR_D_D => OpcodeData {
                name: "OR",
                addr_modes: vec![D, D],
            },
            OR_D_IMM => OpcodeData {
                name: "OR",
                addr_modes: vec![D, Imm],
            },
            OR1_C_NMB => OpcodeData {
                name: "OR1",
                addr_modes: vec![C, Nmb],
            },
            OR1_C_MB => OpcodeData {
                name: "OR1",
                addr_modes: vec![C, Mb],
            },
            PCALL => OpcodeData {
                name: "PCALL",
                addr_modes: vec![Imm],
            },
            POP_A => OpcodeData {
                name: "POP",
                addr_modes: vec![A],
            },
            POP_PSW => OpcodeData {
                name: "POP",
                addr_modes: vec![Psw],
            },
            POP_X => OpcodeData {
                name: "POP",
                addr_modes: vec![X],
            },
            POP_Y => OpcodeData {
                name: "POP",
                addr_modes: vec![Y],
            },
            PUSH_A => OpcodeData {
                name: "PUSH",
                addr_modes: vec![A],
            },
            PUSH_PSW => OpcodeData {
                name: "PUSH",
                addr_modes: vec![Psw],
            },
            PUSH_X => OpcodeData {
                name: "PUSH",
                addr_modes: vec![X],
            },
            PUSH_Y => OpcodeData {
                name: "PUSH",
                addr_modes: vec![Y],
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
                addr_modes: vec![A],
            },
            ROL_D => OpcodeData {
                name: "ROL",
                addr_modes: vec![D],
            },
            ROL_DX => OpcodeData {
                name: "ROL",
                addr_modes: vec![Dx],
            },
            ROL_ABS => OpcodeData {
                name: "ROL",
                addr_modes: vec![Abs],
            },
            ROR_A => OpcodeData {
                name: "ROR",
                addr_modes: vec![A],
            },
            ROR_D => OpcodeData {
                name: "ROR",
                addr_modes: vec![D],
            },
            ROR_DX => OpcodeData {
                name: "ROR",
                addr_modes: vec![Dx],
            },
            ROR_ABS => OpcodeData {
                name: "ROR",
                addr_modes: vec![Abs],
            },
            SBC_IX_IY => OpcodeData {
                name: "SBC",
                addr_modes: vec![Ix, Iy],
            },
            SBC_A_IMM => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Imm],
            },
            SBC_A_IX => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Ix],
            },
            SBC_A_IDY => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Idy],
            },
            SBC_A_IDX => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Idx],
            },
            SBC_A_D => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, D],
            },
            SBC_A_DX => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Dx],
            },
            SBC_A_ABS => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, Abs],
            },
            SBC_A_ABSX => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, AbsX],
            },
            SBC_A_ABSY => OpcodeData {
                name: "SBC",
                addr_modes: vec![A, AbsY],
            },
            SBC_D_D => OpcodeData {
                name: "SBC",
                addr_modes: vec![D, D],
            },
            SBC_D_IMM => OpcodeData {
                name: "SBC",
                addr_modes: vec![D, Imm],
            },
            opcode if opcode & 0x1F == SET1_MASK => OpcodeData {
                name: "SET1",
                addr_modes: vec![Encoded(opcode >> 5), D],
            },
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
                addr_modes: vec![Ya, D],
            },
            o if opcode & 0x0F == TCALL_MASK => {
                let offset = opcode >> 4;
                OpcodeData {
                    name: "TCALL",
                    addr_modes: vec![Encoded(offset)],
                }
            }
            TCLR1_ABS => OpcodeData {
                name: "TCLR1",
                addr_modes: vec![Abs],
            },
            TSET1_ABS => OpcodeData {
                name: "TSET1",
                addr_modes: vec![Abs],
            },
            XCN_A => OpcodeData {
                name: "XCN",
                addr_modes: vec![A],
            },
            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }
}
