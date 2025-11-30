use std::fmt::Display;
use super_yane::Console;
use wdc65816::{format_address_mode, opcode_data};

pub struct InstructionSnapshot {
    pub cpu: wdc65816::Processor,
    pub opcode: u8,
    pub operands: [u8; 3],
}

impl InstructionSnapshot {
    pub fn from(console: &Console) -> Self {
        InstructionSnapshot {
            cpu: console.cpu().clone(),
            opcode: console.opcode(),
            operands: core::array::from_fn(|i| console.read_byte_cpu(console.pc() + 1 + i)),
        }
    }
}
impl Display for InstructionSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = opcode_data(
            self.opcode,
            self.cpu.p.a_is_16bit(),
            self.cpu.p.xy_is_16bit(),
        );
        write!(
            f,
            "{:15} OP={:02X} {:?} (bytes={:02X?}))",
            format!(
                "{} {}",
                data.name,
                format_address_mode(data.addr_mode, &self.operands, data.bytes)
            ),
            self.opcode,
            self.cpu,
            [
                self.opcode,
                self.operands[0],
                self.operands[1],
                self.operands[2]
            ]
        )
    }
}
