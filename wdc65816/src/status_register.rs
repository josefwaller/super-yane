use crate::flag::Flag;

#[derive(Debug, Clone, Copy, Default)]
pub struct StatusRegister {
    /// Carry flag
    pub c: Flag,
    /// Zero flag
    pub z: Flag,
    /// Negative flag
    pub n: Flag,
    /// Decimal mode flag
    pub d: Flag,
    /// Interrupt disable flag
    pub i: Flag,
    /// Memory/Accumulator mode flag
    /// 1 = 8-bit mode, 0 = 16-bit mode
    pub m: Flag,
    /// Overflow flag
    pub v: Flag,
    /// Emulation flag
    pub e: Flag,
    /// Break flag
    pub b: Flag,
}
