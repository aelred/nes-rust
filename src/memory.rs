use crate::Address;

pub trait Memory: Sized {
    fn read(&self, address: Address) -> u8;
    fn write(&mut self, address: Address, byte: u8);
}

pub struct ArrayMemory([u8; 0x10000]);

impl ArrayMemory {
    pub fn slice(&self) -> &[u8] {
        &self.0
    }
}

impl Default for ArrayMemory {
    fn default() -> Self {
        ArrayMemory([0; 0x10000])
    }
}

impl Memory for ArrayMemory {
    fn read(&self, address: Address) -> u8 {
        self.0[address.index()]
    }

    fn write(&mut self, address: Address, byte: u8) {
        self.0[address.index()] = byte;
    }
}

const INTERNAL_RAM_SPACE: Address = Address::new(0x0000);
const PPU_SPACE: Address = Address::new(0x2000);
const PRG_SPACE: Address = Address::new(0x4020);

pub struct NESMemory<PRG> {
    internal_ram: [u8; 0x800],
    prg: PRG,
    the_rest: ArrayMemory, // TODO
}

impl<PRG> NESMemory<PRG> {
    pub fn new(prg: PRG) -> Self {
        NESMemory {
            internal_ram: [0; 0x800],
            prg,
            the_rest: ArrayMemory::default(),
        }
    }
}

impl<PRG: Memory> Memory for NESMemory<PRG> {
    fn read(&self, address: Address) -> u8 {
        if address >= PRG_SPACE {
            self.prg.read(address)
        } else if address >= PPU_SPACE {
            self.the_rest.read(address) // TODO
        } else {
            self.internal_ram[address.index() % 0x0800]
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        if address >= PRG_SPACE {
            self.prg.write(address, byte);
        } else if address >= PPU_SPACE {
            self.the_rest.write(address, byte) // TODO
        } else {
            self.internal_ram[address.index() % 0x0800] = byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_and_write_internal_ram_in_nes_memory() {
        let mut memory = nes_memory();

        for value in 0x0..=0x07ff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
        }
    }

    #[test]
    fn nes_memory_addresses_0x800_to_0x1fff_mirror_internal_ram() {
        let mut memory = nes_memory();

        for value in 0x0800..=0x1fff {
            let address = Address::new(value);
            let true_address = Address::new(value % 0x0800);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.read(true_address), value as u8);

            memory.write(true_address, (value + 1) as u8);
            assert_eq!(memory.read(address), (value + 1) as u8);
        }
    }

    #[test]
    fn can_read_and_write_cartridge_space_in_nes_memory() {
        let mut memory = nes_memory();

        for value in 0x4020..=0xffff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.prg.read(address), value as u8);
        }
    }

    fn nes_memory() -> NESMemory<ArrayMemory> {
        let prg = ArrayMemory::default();
        NESMemory::new(prg)
    }
}