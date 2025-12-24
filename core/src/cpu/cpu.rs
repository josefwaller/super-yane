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
        let current_dma =
            (0..memory.dma_channels.len()).find(|i| memory.dma_channels[*i].is_executing);
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
                let v = memory.read(src);
                memory.write(dest, v);
                if i == 4 {
                    debug!(
                        "Copy {} {:02X} from {:04X} to {:04X} ({:?}) lc={}",
                        i,
                        v,
                        src,
                        dest,
                        (memory.ppu.cursor_x(), memory.ppu.cursor_y()),
                        d.hdma_line_counter
                    );
                }
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
    pub fn on_irq(&mut self, memory: &mut impl HasAddressBus) {
        self.core.on_irq(memory);
    }
    pub fn reset(&mut self, memory: &mut impl HasAddressBus) {
        self.core.reset(memory);
    }
}
