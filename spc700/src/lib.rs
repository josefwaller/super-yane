mod opcode_data;
mod opcodes;
mod processor;
mod program_status_word;

/// The Initial Program Load ROM
pub const IPL: &[u8; 64] = include_bytes!("./ipl.bin");
pub use opcode_data::{AddressMode, OpcodeData, format_address_modes};
pub use processor::*;
pub use program_status_word::ProgramStatusWord;
