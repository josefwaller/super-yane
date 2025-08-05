use std::fmt::Display;

use log::*;

#[derive(Debug, Clone, Copy)]
pub struct StatusRegister {
    /// Carry flag
    pub c: bool,
    /// Zero flag
    pub z: bool,
    /// Negative flag
    pub n: bool,
    /// Decimal mode flag
    pub d: bool,
    /// Interrupt disable flag
    pub i: bool,
    /// Memory/Accumulator mode flag
    /// 1 = 8-bit mode, 0 = 16-bit mode
    pub m: bool,
    /// Overflow flag
    pub v: bool,
    /// Emulation flag
    /// 0 = native (16-bit), 1 = emulation (8-bit)
    pub e: bool,
    /// Break flag or Index register width flag
    /// 1 = 8-bit, 0=16-bit
    pub xb: bool,
}
impl Default for StatusRegister {
    fn default() -> Self {
        StatusRegister {
            c: false,
            z: false,
            n: false,
            d: false,
            i: false,
            m: false,
            v: false,
            e: true,
            xb: false,
        }
    }
}

impl StatusRegister {
    pub fn a_is_8bit(&self) -> bool {
        self.m || self.e
    }
    pub fn a_is_16bit(&self) -> bool {
        !self.a_is_8bit()
    }
    pub fn xy_is_8bit(&self) -> bool {
        self.xb || self.e
    }
    pub fn xy_is_16bit(&self) -> bool {
        !self.xy_is_8bit()
    }
    pub fn from_byte(byte: u8, e: bool) -> StatusRegister {
        macro_rules! bit {
            ($bit_num: expr) => {
                ((byte >> $bit_num) & 0x01) == 1
            };
        }
        StatusRegister {
            c: bit!(0),
            z: bit!(1),
            i: bit!(2),
            d: bit!(3),
            // The M and X flags are forced to 1 if E is 1
            xb: bit!(4),
            m: bit!(5),
            v: bit!(6),
            n: bit!(7),
            e,
        }
    }
    pub fn to_byte(&self, force_bytes: bool) -> u8 {
        let mut value = 0;

        macro_rules! set_bit {
            ($bit_num: expr, $value: expr) => {
                if $value {
                    value |= (0x01 << $bit_num)
                }
            };
        }
        set_bit!(0, self.c);
        set_bit!(1, self.z);
        set_bit!(2, self.i);
        set_bit!(3, self.d);
        set_bit!(4, self.xb || (force_bytes && self.e));
        set_bit!(5, self.m || (force_bytes && self.e));
        set_bit!(6, self.v);
        set_bit!(7, self.n);

        value
    }
}

impl Display for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X} (C:{} Z:{} I:{} D:{} M:{} V:{} N:{} E:{} XB:{})",
            self.to_byte(false),
            if self.c { "1" } else { "0" },
            if self.z { "1" } else { "0" },
            if self.i { "1" } else { "0" },
            if self.d { "1" } else { "0" },
            if self.m { "1" } else { "0" },
            if self.v { "1" } else { "0" },
            if self.n { "1" } else { "0" },
            if self.e { "1" } else { "0" },
            if self.xb { "1" } else { "0" }
        )
    }
}
