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
    pub e: bool,
    /// Break flag
    pub b: bool,
}

impl StatusRegister {
    pub fn is_8bit(&self) -> bool {
        self.m.into()
    }
    pub fn is_16bit(&self) -> bool {
        !self.is_8bit()
    }
}
