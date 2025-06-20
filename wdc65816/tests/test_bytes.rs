use serde::{Deserialize, Serialize};
use wdc65816::{HasAddressBus, Processor};

#[derive(Serialize, Deserialize, Debug)]
struct OpcodeData {
    code: u8,
    name: String,
    addr_mode: String,
    desc: String,
    addr_mode_desc: String,
    bytes: Vec<u16>,
}

// Memory that just points to one value
struct OpcodeMemory {
    opcode: u8,
}

impl HasAddressBus for OpcodeMemory {
    fn io(&mut self) {}
    fn read(&mut self, _address: usize) -> u8 {
        self.opcode
    }
    fn write(&mut self, _address: usize, _value: u8) {}
}
const OPCODE_DATA_STR: &str = include_str!("../opcode_data.json");

#[test]
fn test_byte_length() {
    let all_opcodes: Vec<OpcodeData> = serde_json::from_str(OPCODE_DATA_STR).unwrap();
    let blacklist_ops = vec![
        "Branch",
        "Jump",
        "Return",
        "BRK",
        "COP",
        "Stop",
        "Wait",
        "Block Move",
        "Reserved",
    ];
    all_opcodes.iter().for_each(|f| {
        if blacklist_ops
            .iter()
            .find(|op| f.desc.contains(*op))
            .is_some()
        {
            return;
        }
        let mut processor = Processor::new();
        let mut memory = OpcodeMemory { opcode: f.code };
        processor.step(&mut memory);
        assert!(
            f.bytes.contains(&processor.pc),
            "{} {} should advance PC by {:?} bytes, but PC was {:X}",
            f.name,
            f.addr_mode,
            f.bytes,
            &processor.pc
        );
    });
}
