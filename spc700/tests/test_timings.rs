use serde::Deserialize;
use spc700::opcodes::*;
use spc700::{HasAddressBus, Processor};

/// Very simple memory used for tests
struct Mem {
    ram: [u8; 100],
}

impl Mem {
    fn new(start_mem: &[u8]) -> Mem {
        let mut m = Mem { ram: [0; 100] };
        m.ram[0..start_mem.len()].copy_from_slice(start_mem);
        m
    }
}

impl HasAddressBus for Mem {
    fn io(&mut self) {}
    fn read(&mut self, address: usize) -> u8 {
        self.ram[address % self.ram.len()]
    }
    fn write(&mut self, address: usize, value: u8) {
        self.ram[address % self.ram.len()] = value;
    }
}

#[derive(Deserialize)]
struct OpcodeInfo {
    code: u8,
    bytes: Option<u8>,
    cycles: Option<usize>,
}

/// Test all opcodes have the correct length
#[test]
fn test_opcode_lengths() -> Result<(), serde_json::Error> {
    let data: Vec<OpcodeInfo> = serde_json::from_str(include_str!("./opcode_data.json"))?;
    for d in data.iter() {
        if let Some(b) = d.bytes {
            let mut p = Processor::default();
            p.pc = 0;
            let mut mem = Mem::new(&[d.code]);
            p.step(&mut mem);
            if d.code == BRK {
                p.execute_opcode(RETI, &mut mem);
            } else if [CALL_ABS, PCALL].contains(&d.code) || d.code & 0x0F == TCALL_MASK {
                p.execute_opcode(RET, &mut mem);
            } else if [JMP_ABS, JMP_IAX, RET, RETI, SLEEP, STOP].contains(&d.code) {
                // Skip jumps and returns
                continue;
            }
            assert_eq!(
                p.pc as u8, b,
                "Opcode {:02X} has incorrect length {} (should be {})",
                d.code, p.pc, b
            );
        }
    }
    Ok(())
}
