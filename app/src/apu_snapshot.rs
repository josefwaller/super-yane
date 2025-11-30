use spc700::{OpcodeData as Spc700OpcodeData, format_address_modes};
use std::fmt::Display;
use super_yane::Console;

pub struct ApuSnapshot {
    pub cpu: spc700::Processor,
    pub opcode: u8,
    pub operands: [u8; 3],
}

impl ApuSnapshot {
    pub fn from(console: &Console) -> Self {
        let apu = console.apu().core;
        ApuSnapshot {
            cpu: apu.clone(),
            opcode: console.apu().read_ram(apu.pc as usize),
            operands: core::array::from_fn(|i| {
                console
                    .apu()
                    .read_ram(apu.pc.wrapping_add(1 + i as u16) as usize)
            }),
        }
    }
}

impl Display for ApuSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = Spc700OpcodeData::from_opcode(self.opcode);
        write!(
            f,
            "{:20} OP={:02X} {} (bytes={:02X?})",
            format!(
                "{} {}",
                data.name,
                format_address_modes(&data.addr_modes, &self.operands)
            ),
            self.opcode,
            self.cpu,
            self.operands
        )
    }
}
