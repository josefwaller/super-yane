use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Window {
    pub left: usize,
    pub right: usize,
    pub enabled_color: bool,
    pub invert_color: bool,
    /// Window enabled for sprites/OAM
    #[serde(default)]
    pub enabled_sprite: bool,
    /// Window inverted for sprites/OAM
    #[serde(default)]
    pub invert_sprite: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
