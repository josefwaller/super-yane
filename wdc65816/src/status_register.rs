#[derive(Debug, Clone, Copy, Default)]
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

impl StatusRegister {
    pub fn a_is_8bit(&self) -> bool {
        self.m
    }
    pub fn a_is_16bit(&self) -> bool {
        !self.a_is_8bit()
    }
    pub fn xy_is_8bit(&self) -> bool {
        self.e || self.xb
    }
    pub fn xy_is_16bit(&self) -> bool {
        !self.e && !self.xb
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
            xb: bit!(4) || e,
            m: bit!(5) || e,
            v: bit!(6),
            n: bit!(7),
            e,
        }
    }
    pub fn to_byte(&self) -> u8 {
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
        set_bit!(4, self.xb);
        set_bit!(5, self.m);
        set_bit!(6, self.v);
        set_bit!(7, self.n);

        value
    }
}
