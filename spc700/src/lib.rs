mod opcode_data;
mod opcodes;
mod processor;
mod status_register;

/// The Initial Program Load ROM
pub const IPL: &[u8; 64] = include_bytes!("./ipl.bin");
pub use opcode_data::{AddressMode, OpcodeData};
pub use processor::*;
pub use status_register::StatusRegister;
