use serde::{Deserialize, Serialize};

// Struct for the SNES multiplication and division registers
#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Math {
    // Multiplication values
    m_a: u8,
    m_b: u8,
    // Multiplication result
    m_r: u16,
    /// Dividend, as LE bytes
    dividend: [u8; 2],
    /// Divisor
    divisor: u8,
    /// Division result, as LE bytes
    div_res: [u8; 2],
    /// Product or remainder, as LE bytes
    product_remainder: [u8; 2],
}

impl Math {
    pub fn write_byte(&mut self, address: usize, value: u8) {
        match address {
            0x4202 => {
                self.m_a = value;
            }
            0x4203 => {
                self.m_b = value;
                self.product_remainder = (self.m_a as u16 * self.m_b as u16).to_le_bytes();
                self.div_res[0] = self.m_b;
                self.div_res[1] = 0;
            }
            0x4204 => self.dividend[0] = value,
            0x4205 => self.dividend[1] = value,
            0x4206 => {
                self.divisor = value;
                let dividend = u16::from_le_bytes(self.dividend);
                let (div_res, remainder) = if value == 0 {
                    (0xFFFF, dividend)
                } else {
                    (dividend / value as u16, dividend % value as u16)
                };
                self.div_res = div_res.to_le_bytes();
                self.product_remainder = remainder.to_le_bytes();
            }
            _ => {}
        }
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        match address {
            0x4214 => self.div_res[0],
            0x4215 => self.div_res[1],
            0x4216 => self.product_remainder[0],
            0x4217 => self.product_remainder[1],
            _ => 0,
        }
    }
}
