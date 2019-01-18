use crate::Address;

pub trait Memory: Sized {
    /// This method takes a mutable reference because reading from memory can sometimes trigger
    /// state changes.
    ///
    /// e.g. when reading from the PPU status register, bit 7 of the register is reset.
    fn read(&mut self, address: Address) -> u8;
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
    fn read(&mut self, address: Address) -> u8 {
        self.0[address.index()]
    }

    fn write(&mut self, address: Address, byte: u8) {
        self.0[address.index()] = byte;
    }
}

impl<'a, T: Memory> Memory for &'a mut T {
    fn read(&mut self, address: Address) -> u8 {
        T::read(self, address)
    }

    fn write(&mut self, address: Address, byte: u8) {
        T::write(self, address, byte)
    }
}

const PPU_SPACE: Address = Address::new(0x2000);
const PRG_SPACE: Address = Address::new(0x4020);

pub struct NESCPUMemory<PRG> {
    internal_ram: [u8; 0x800],
    prg: PRG,
    the_rest: ArrayMemory, // TODO
}

impl<PRG> NESCPUMemory<PRG> {
    pub fn new(prg: PRG) -> Self {
        NESCPUMemory {
            internal_ram: [0; 0x800],
            prg,
            the_rest: ArrayMemory::default(),
        }
    }
}

impl<PRG: Memory> Memory for NESCPUMemory<PRG> {
    fn read(&mut self, address: Address) -> u8 {
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

const CHR_END: usize = PALETTE_OFFSET - 1;
const PALETTE_OFFSET: usize = 0x3f00;

pub struct NESPPUMemory<CHR> {
    palette_ram: [u8; 0x20],
    chr: CHR,
}

impl<CHR> NESPPUMemory<CHR> {
    pub fn new(chr: CHR) -> Self {
        let mut palette_ram = [0; 0x20];

        for i in 0..0x20 {
            palette_ram[i] = (i * 4) as u8;
        }

        NESPPUMemory {
            palette_ram,
            chr,
        }
    }
}

impl<CHR: Memory> Memory for NESPPUMemory<CHR> {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x0000...CHR_END => self.chr.read(address),
            PALETTE_OFFSET...0x3f1f => self.palette_ram[address.index() - PALETTE_OFFSET],
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x0000...CHR_END => self.chr.write(address, byte),
            PALETTE_OFFSET...0x3f1f => self.palette_ram[address.index() - PALETTE_OFFSET] = byte,
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_and_write_internal_ram_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        for value in 0x0..=0x07ff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
        }
    }

    #[test]
    fn nes_cpu_memory_addresses_0x800_to_0x1fff_mirror_internal_ram() {
        let mut memory = nes_cpu_memory();

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
    fn can_read_and_write_cartridge_space_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        for value in 0x4020..=0xffff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.prg.read(address), value as u8);
        }
    }

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
                memory.palette_ram[address.index() - 0x3f00],
                (value + 1) as u8
            );
        }
    }

    fn nes_cpu_memory() -> NESCPUMemory<ArrayMemory> {
        let prg = ArrayMemory::default();
        NESCPUMemory::new(prg)
    }

    fn nes_ppu_memory() -> NESPPUMemory<ArrayMemory> {
        let chr = ArrayMemory::default();
        NESPPUMemory::new(chr)
    }
}
