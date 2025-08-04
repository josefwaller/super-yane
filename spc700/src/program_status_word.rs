#[derive(Default, Copy, Clone, Debug)]
pub struct ProgramStatusWord {
    pub n: bool,
    pub v: bool,
    pub p: bool,
    pub b: bool,
    pub h: bool,
    pub i: bool,
    pub z: bool,
    pub c: bool,
}
impl ProgramStatusWord {
    pub fn to_byte(&self) -> u8 {
        macro_rules! bit {
            ($num: expr, $flag: ident) => {{ if self.$flag { (1 << $num) } else { 0 } }};
        }
        bit!(7, n)
            | bit!(6, v)
            | bit!(5, p)
            | bit!(4, b)
            | bit!(3, h)
            | bit!(2, i)
            | bit!(1, z)
            | bit!(0, c)
    }
    pub fn from_byte(byte: u8) -> ProgramStatusWord {
        macro_rules! bit {
            ($num: expr) => {{ ((byte >> $num) & 0x01) != 0 }};
        }
        ProgramStatusWord {
            n: bit!(7),
            v: bit!(6),
            p: bit!(5),
            b: bit!(4),
            h: bit!(3),
            i: bit!(2),
            z: bit!(1),
            c: bit!(0),
        }
    }
}
