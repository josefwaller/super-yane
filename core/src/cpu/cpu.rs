use crate::console::ExternalArchitecture;
use log::*;
use serde::{Deserialize, Serialize};
use wdc65816::{HasAddressBus, Processor as WdcProcessor};

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct Cpu {
    pub(crate) core: WdcProcessor,
}

impl Cpu {
    pub fn step(&mut self, memory: &mut ExternalArchitecture) {
        let x = memory.ppu.dot_xy().0;
        if x > 134 && x < 144 {
            // Pause for 40 cycles
            memory.advance(40);
            return;
        }
        // Prioritize HDMA
        let current_dma = (0..memory.dma_channels.len())
            .find(|i| memory.dma_channels[*i].is_executing && memory.dma_channels[*i].is_hdma())
            .or((0..memory.dma_channels.len()).find(|i| memory.dma_channels[*i].is_executing));
        match current_dma {
            Some(i) => {
                // Temporarily clone
                let mut d = memory.dma_channels[i].clone();
                // Get addresses to read to/from
                let src = d.full_src_addr();
                let dest = d.dest_addr
                    + d.transfer_pattern()
                        [d.num_bytes_transferred as usize % d.transfer_pattern().len()]
                        as usize;
                // Transfer
                // Ignore the timing since DMA is always 8 cycles per byte
                if d.direction {
                    let v = memory.read_byte(dest).0;
                    memory.write_byte(src, v);
                } else {
                    let v = memory.read_byte(src).0;
                    memory.write_byte(dest, v);
                }
                // Advance 8 cycles per byte
                memory.advance(8);
                // Increment source address
                d.inc_src_addr();
                // Decrement byte counter
                d.num_bytes_transferred = d.num_bytes_transferred.wrapping_add(1);
                if d.num_bytes_transferred == d.get_num_bytes() {
                    d.is_executing = false;
                    d.byte_counter = 0;
                }
                memory.dma_channels[i] = d;
            }
            None => self.core.step(memory),
        }
    }
    pub fn on_nmi(&mut self, memory: &mut impl HasAddressBus) {
        self.core.on_nmi(memory);
    }
    pub fn on_irq(&mut self, memory: &mut impl HasAddressBus) {
        self.core.on_irq(memory);
    }
    pub fn reset(&mut self, memory: &mut impl HasAddressBus) {
        self.core.reset(memory);
    }
}
