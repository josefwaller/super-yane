use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum MemoryMap {
    LoRom,
    HiRom,
    ExHiRom,
}

impl MemoryMap {
    pub fn transform_address(&self, address: usize) -> usize {
        match self {
            MemoryMap::LoRom => (address & 0x7FFF) + ((address >> 1) & 0x7F_8000),
            _ => todo!("{:?} memory mapping", self),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Cartridge {
    memory_map: MemoryMap,
    data: Vec<u8>,
    sram: Vec<u8>,
}

impl Cartridge {
    pub fn from_data(data: &[u8]) -> Cartridge {
        let memory_map = {
            // Figure out what kind of cartridge based on the checksum
            let checksum = if data.len().is_power_of_two() {
                // Cartridge is already a power of 2
                data.iter().fold(0u16, |acc, b| acc.wrapping_add(*b as u16))
            } else {
                let lesser = data.len().ilog2();
                debug!("Lesser is {:X}", lesser);
                let remainder = &data[2usize.pow(lesser)..data.len()];
                debug!("Remainder size is {:X}", remainder.len());
                // Get the next largest power of two
                let greater = remainder.len().ilog2()
                    + if remainder.len().is_power_of_two() {
                        0
                    } else {
                        1
                    };
                let mut count = 0;
                std::iter::from_fn(|| {
                    let r = if count > 2usize.pow(lesser + 1) {
                        // Size is a power of two
                        None
                    } else if count < 2usize.pow(lesser) {
                        // Return initial part of data
                        Some(data[count])
                    } else {
                        let i = (count - 2usize.pow(lesser)) % 2usize.pow(greater);
                        if i > remainder.len() {
                            // Essentially just pad with 0s
                            Some(0)
                        } else {
                            // Return data
                            Some(remainder[i])
                        }
                    };
                    count += 1;
                    r
                })
                .fold(0u16, |acc, e| acc.wrapping_add(e as u16))
            };
            debug!("Checksum is {:X} {:X}", checksum, checksum ^ 0xFFFF);
            MemoryMap::LoRom
        };
        let sram_len = {
            let n = data[(memory_map.transform_address(0x00FFD8)) % data.len()];
            (1 << n) * 1024
        };
        debug!("SRAM len: {}", sram_len);
        debug!(
            "Country is {}",
            data[memory_map.transform_address(0x00FFD9) % data.len()]
        );
        Cartridge {
            data: match memory_map {
                MemoryMap::LoRom => data.to_vec(),
                _ => vec![],
            },
            sram: vec![0; sram_len],
            memory_map,
        }
    }
    pub fn transform_address(&self, address: usize) -> usize {
        self.memory_map.transform_address(address)
    }
    pub fn write_byte(&mut self, address: usize, value: u8) {
        if (70_0000..0x7E_0000).contains(&(address % 0x80_0000)) && (address & 0xFFFF) < 0x8000 {
            let i = address % self.sram.len();
            self.sram[i] = value;
        }
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        if (70_0000..0x7E_0000).contains(&(address % 0x80_0000)) && (address & 0xFFFF) < 0x8000 {
            self.sram[address % self.sram.len()]
        } else {
            self.data[self.transform_address(address) % self.data.len()]
        }
    }
    // TBD: Should there just be a `header` method that returns a custom struct?
    pub fn title(&self) -> String {
        (0..21)
            .map(|i| self.read_byte(0xFFC0 + i) as char)
            .collect()
    }
}
