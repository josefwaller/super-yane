use log::debug;

#[derive(Clone, Copy)]
enum MemoryMap {
    LoRom,
    HiRom,
    ExHiRom,
}

#[derive(Clone)]
pub struct Cartridge {
    memory_map: MemoryMap,
    data: Vec<u8>,
}

impl Cartridge {
    pub fn from_data(data: &[u8]) -> Cartridge {
        debug!("{:X}", data.len());
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
        Cartridge {
            data: match memory_map {
                MemoryMap::LoRom => data.to_vec(),
                _ => vec![],
            },
            memory_map,
        }
    }
    pub fn transform_address(&self, address: usize) -> usize {
        match self.memory_map {
            MemoryMap::LoRom => address & 0x7F7FFF,
            _ => 0,
        }
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        self.data[self.transform_address(address)]
    }
}
