use crate::{console::ExternalArchitecture, dma::AddressAdjustMode as DmaAddressAdjustMode};
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
                // This macro just exists to get around borrowing memory as mutable twice.
                // Once for the DMA register, and once for the read/write calls.
                macro_rules! d {
                    () => {
                        memory.dma_channels[i]
                    };
                }
                let bytes_transferred =
                    d!().init_byte_counter.wrapping_sub(d!().byte_counter) as usize;
                // Todo: handling timing of DMA
                let src = d!().src_bank as usize * 0x10000 + d!().src_addr as usize;
                let dest = d!().dest_addr
                    + d!().transfer_pattern[bytes_transferred % d!().transfer_pattern.len()]
                        as usize;
                let v = memory.read(src);
                memory.write(dest, v);
                match d!().adjust_mode {
                    DmaAddressAdjustMode::Increment => {
                        d!().src_addr = d!().src_addr.wrapping_add(1)
                    }
                    DmaAddressAdjustMode::Decrement => {
                        d!().src_addr = d!().src_addr.wrapping_sub(1)
                    }
                    _ => {}
                }
                d!().byte_counter = d!().byte_counter.wrapping_sub(1);
                if d!().byte_counter == 0 {
                    d!().is_executing = false;
                }
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
