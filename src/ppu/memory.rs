use crate::Address;
use crate::Memory;

const CHR_END: usize = PALETTE_OFFSET - 1;
const PALETTE_OFFSET: usize = 0x3f00;

pub struct NESPPUMemory<CHR> {
    palette_ram: [u8; 0x20],
    chr: CHR,
}

impl<CHR> NESPPUMemory<CHR> {
    pub fn new(chr: CHR) -> Self {
        let palette_ram = [
            0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00, 0x04, 0x2C,
            0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02, 0x00, 0x20, 0x2C, 0x08
        ];

        NESPPUMemory { palette_ram, chr }
    }

    fn palette_index(&self, address: Address) -> usize {
        let mut index = (address.index() - PALETTE_OFFSET) % 0x0020;

        let is_unused_colour = index % 0x04 == 0;

        // unused colours mirror between sprite and background palettes
        if is_unused_colour {
            index &= 0b1111;
        }

        index
    }
}

impl<CHR: Memory> Memory for NESPPUMemory<CHR> {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x0000..=CHR_END => self.chr.read(address),
            PALETTE_OFFSET..=0x3fff => self.palette_ram[self.palette_index(address)],
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x0000..=CHR_END => self.chr.write(address, byte),
            PALETTE_OFFSET..=0x3fff => {
                self.palette_ram[self.palette_index(address)] = byte;
            }
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ArrayMemory;

    use super::*;

    #[test]
    fn can_read_cartridge_space_in_nes_ppu_memory() {
        let mut memory = nes_ppu_memory();

        for value in 0x0000..=0x3eff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.chr.read(address), value as u8);
        }
    }

    #[test]
    fn can_read_palette_ram_in_nes_ppu_memory() {
        let mut memory = nes_ppu_memory();

        for value in 0x3f00..=0x3f1f {
            let address = Address::new(value);

            memory.write(address, (value + 1) as u8);
            assert_eq!(memory.read(address), (value + 1) as u8);
            assert_eq!(
                memory.palette_ram[memory.palette_index(address)],
                (value + 1) as u8
            );
        }
    }

    #[test]
    fn palette_ram_mirrors_from_0x3f20_to_0x3fff() {
        let mut memory = nes_ppu_memory();

        for value in 0x3f20..=0x3fff {
            let address = Address::new(value);

            memory.write(address, (value + 1) as u8);
            assert_eq!(memory.read(address), (value + 1) as u8);
            assert_eq!(
                memory.palette_ram[memory.palette_index(address)],
                (value + 1) as u8
            );
        }
    }

    #[test]
    fn palette_ram_mirrors_0x3f1x_to_0x3f0x_for_0_4_8_and_c() {
        let mut memory = nes_ppu_memory();

        let addresses = [
            (0x3f00, 0x3f10),
            (0x3f04, 0x3f14),
            (0x3f08, 0x3f18),
            (0x3f0c, 0x3f1c),
        ];

        for (original, mirror) in addresses.iter() {
            let original_address = Address::new(*original);
            let mirror_address = Address::new(*mirror);

            memory.write(original_address, 42);
            assert_eq!(memory.read(mirror_address), 42);
            memory.write(mirror_address, 24);
            assert_eq!(memory.read(original_address), 24);
        }
    }

    fn nes_ppu_memory() -> NESPPUMemory<ArrayMemory> {
        let chr = ArrayMemory::default();
        NESPPUMemory::new(chr)
    }
}
