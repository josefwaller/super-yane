use wdc65816::{HasAddressBus, Processor};

pub struct Console {
    pub cpu: Processor,
    pub ram: [u8; 0x2000],
}

macro_rules! wrapper {
    ($self: ident) => {{
        struct Wrapper<'a> {
            ram: &'a mut [u8; 0x2000],
        }
        impl HasAddressBus for Wrapper<'_> {
            fn io(&mut self) {}
            fn read(&self, address: usize) -> u8 {
                self.ram[address % 0x20000]
            }
            fn write(&mut self, address: usize, value: u8) {
                self.ram[address % 0x20000] = value;
            }
        }
        Wrapper {
            ram: &mut $self.ram,
        }
    }};
}

impl Console {
    pub fn advance_instructions(&mut self, num_instructions: u32) {
        let mut wrapper = wrapper!(self);
        (0..num_instructions).for_each(|_| self.cpu.step(&mut wrapper))
    }
    pub fn advance_until(&mut self, should_stop: impl Fn(&Console) -> bool) -> u32 {
        std::iter::from_fn(|| {
            if should_stop(&self) {
                None
            } else {
                let mut wrapper = wrapper!(self);
                self.cpu.step(&mut wrapper);
                Some(1)
            }
        })
        .sum()
    }
}
