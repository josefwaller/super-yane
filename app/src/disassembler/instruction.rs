use super_yane::Console;
use wdc65816::{OpcodeData, format_address_mode, opcode_data};

#[derive(Clone, Copy)]
pub struct Instruction {
    opcode: u8,
    operands: [u8; 3],
    a: bool,
    xy: bool,
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        let data = opcode_data(self.opcode, self.a, self.xy);
        format!(
            "{} {}",
            data.name,
            format_address_mode(data.addr_mode, &self.operands, data.bytes)
        )
    }
}

impl Instruction {
    pub fn from_console(value: &Console) -> Self {
        Instruction {
            opcode: value.opcode(),
            operands: core::array::from_fn(|i| value.read_byte_cpu(value.pc() + i + 1)),
            a: value.cpu().p.a_is_16bit(),
            xy: value.cpu().p.xy_is_16bit(),
        }
    }
    pub fn data(&self) -> OpcodeData {
        opcode_data(self.opcode, self.a, self.xy)
    }
    pub fn operands(&self) -> &[u8] {
        &self.operands[0..(self.data().bytes as usize)]
    }
}
