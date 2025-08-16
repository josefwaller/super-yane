use crate::console::ExternalArchitecture;
use log::*;
use wdc65816::{HasAddressBus, Processor as WdcProcessor};

#[derive(Default, Copy, Clone)]
pub struct Cpu {
    pub(crate) core: WdcProcessor,
}

impl Cpu {
    pub fn step(&mut self, memory: &mut ExternalArchitecture) {
        let current_dma =
            (0..memory.dma_channels.len()).find(|i| memory.dma_channels[*i].is_executing);
        match current_dma {
            Some(i) => {
                // Temporarily clone
                let mut d = memory.dma_channels[i].clone();
                // Get addresses to read to/from
                let src = d.full_src_addr();
                let dest = d.dest_addr
                    + d.transfer_pattern
                        [d.num_bytes_transferred as usize % d.transfer_pattern.len()]
                        as usize;
                // Transfer
                let v = memory.read(src);
                memory.write(dest, v);
                // Increment source address
                d.inc_src_addr();
                // Decrement byte counter
                d.num_bytes_transferred = d.num_bytes_transferred.wrapping_add(1);
                if d.num_bytes_transferred == d.get_num_bytes() {
                    d.is_executing = false;
                }
                memory.dma_channels[i] = d;
            }
            None => self.core.step(memory),
        }
    }
    pub fn on_nmi(&mut self, memory: &mut impl HasAddressBus) {
        self.core.on_nmi(memory);
    }
    pub fn reset(&mut self, memory: &mut impl HasAddressBus) {
        self.core.reset(memory);
    }
}
