use wdc65816::{HasAddressBus, Processor as WdcProcessor};

#[derive(Default, Copy, Clone)]
pub struct Cpu {
    pub(crate) core: WdcProcessor,
}

impl Cpu {
    pub fn step(&mut self, memory: &mut impl HasAddressBus) {
        self.core.step(memory);
    }
    pub fn on_nmi(&mut self, memory: &mut impl HasAddressBus) {
        self.core.on_nmi(memory);
    }
    pub fn reset(&mut self, memory: &mut impl HasAddressBus) {
        self.core.reset(memory);
    }
}
