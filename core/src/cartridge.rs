use log::{debug, error};
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
            MemoryMap::HiRom => address & 0x3F_FFFF,
            MemoryMap::ExHiRom => (((!address) & 0x80_0000) >> 1) | (address & 0x3F_FFFF),
        }
    }
    pub fn is_sram_address(&self, address: usize) -> bool {
        match self {
            MemoryMap::LoRom => {
                (0x70_0000..0x7E_0000).contains(&(address % 0x80_0000))
                    && (address & 0xFFFF) < 0x8000
            }
            MemoryMap::HiRom => {
                (0x30_0000..0x40_0000).contains(&address)
                    && (0x6000..0x8000).contains(&(address & 0xFFFF))
            }
            MemoryMap::ExHiRom => {
                (0x80_0000..0xC0_0000).contains(&address)
                    && (0x6000..0x8000).contains(&(address & 0xFFFF))
            }
        }
    }
}

fn u16_at(arr: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([arr[i], arr[i + 1]])
}

fn checksum_matches(data: &[u8], header_index: usize, checksum: u16) -> bool {
    // Verify length
    if header_index + 0x40 > data.len() {
        false
    } else {
        if checksum == u16_at(data, header_index + 0x1E)
            && checksum ^ 0xFFFF == u16_at(data, header_index + 0x1C)
        {
            true
        } else {
            false
        }
    }
}

fn compute_checksum(data: &[u8]) -> u16 {
    if data.len().is_power_of_two() {
        // Cartridge is already a power of 2
        debug!("Cartridge size is power of 2");
        data.iter().fold(0u16, |acc, b| acc.wrapping_add(*b as u16))
    } else {
        debug!("Cartridge is not a power of 2");
        // Get largest power of two that is smaller than the data length
        let lesser = data.len().ilog2();
        let index = 2usize.pow(lesser);
        // Get the smallest power of two that is largest than the remainder length
        let remainder_len = data.len() - index;
        let greater = remainder_len.ilog2()
            + if remainder_len.is_power_of_two() {
                0
            } else {
                1
            };
        // Compute checksums
        let larger_checksum = &data[0..index]
            .iter()
            .fold(0u16, |acc, b| acc.wrapping_add(*b as u16));
        let smaller_checksum = &data[index..data.len()]
            .iter()
            .fold(0u16, |acc, b| acc.wrapping_add(*b as u16));
        // Combine as if repeating the remainder checksum to get an array that is 2 * index long
        larger_checksum
            .wrapping_add(smaller_checksum.wrapping_mul((index / 2usize.pow(greater)) as u16))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Cartridge {
    memory_map: MemoryMap,
    pub data: Vec<u8>,
    sram: Vec<u8>,
}

impl Cartridge {
    pub fn from_data(data: &[u8]) -> Cartridge {
        let (memory_map, has_header) = {
            // Figure out what kind of cartridge based on the checksum
            let (checksum, header_checksum) =
                (compute_checksum(data), compute_checksum(&data[0x200..]));
            debug!("Checksum: {:04X}", checksum);
            if checksum_matches(data, 0x007FC0, checksum) {
                (MemoryMap::LoRom, false)
            } else if checksum_matches(data, 0x00FFC0, checksum) {
                (MemoryMap::HiRom, false)
            } else if checksum_matches(data, 0x40FFC0, checksum) {
                (MemoryMap::ExHiRom, false)
            } else if checksum_matches(data, 0x007FC0 + 0x200, header_checksum) {
                (MemoryMap::LoRom, true)
            } else if checksum_matches(data, 0x00FFC0 + 0x200, header_checksum) {
                (MemoryMap::HiRom, true)
            } else {
                error!("Unable to determine ROM memory map. Defaulting to LoRom");
                (MemoryMap::LoRom, false)
            }
        };
        debug!("Memory map detected as {:?}", memory_map);
        let data = if has_header {
            data[512..].to_vec()
        } else {
            data.to_vec()
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
            data,
            sram: vec![0; sram_len],
            memory_map,
        }
    }
    pub fn transform_address(&self, address: usize) -> usize {
        self.memory_map.transform_address(address)
    }
    pub fn write_byte(&mut self, address: usize, value: u8) {
        if self.memory_map.is_sram_address(address) {
            let i = address % self.sram.len();
            self.sram[i] = value;
        }
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        if self.memory_map.is_sram_address(address) {
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
