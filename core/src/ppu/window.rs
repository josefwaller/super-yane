#[derive(Default, Debug, Clone, Copy)]
pub struct Window {
    pub left: usize,
    pub right: usize,
    pub enabled_color: bool,
    pub invert_color: bool,
}

#[derive(Clone, Copy)]
pub enum WindowRegion {
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}
impl From<u8> for WindowRegion {
    fn from(value: u8) -> Self {
        use WindowRegion::*;
        match value & 003 {
            0 => Nowhere,
            1 => Outside,
            2 => Inside,
            3 => Everywhere,
            _ => unreachable!(),
        }
    }
}
impl WindowRegion {
    pub fn compute(&self, val: bool) -> bool {
        use WindowRegion::*;
        match self {
            Nowhere => false,
            Everywhere => true,
            Inside => val,
            Outside => !val,
        }
    }
}
