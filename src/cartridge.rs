use crate::mapper::Mapper;
use crate::Address;
use crate::Memory;

pub struct Cartridge {
    prg_rom: Box<[u8]>,
    prg_ram: [u8; 0x2000],
    mapper: Mapper,
}

impl Cartridge {
    pub fn new(prg_rom: Box<[u8]>, mapper: Mapper) -> Self {
        if mapper != Mapper::NROM {
            unimplemented!("Unsupported mapper {:?}", mapper);
        }

        Cartridge {
            prg_rom,
            prg_ram: [0; 0x2000],
            mapper,
        }
    }
}

impl Memory for Cartridge {
    fn read(&self, address: Address) -> u8 {
        match address.index() {
            0x6000...0x7fff => self.prg_ram[address.index() - 0x6000],
            0x8000...0xffff => self.prg_rom[address.index() - 0x8000],
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x6000...0x7fff => {
                self.prg_ram[address.index() - 0x6000] = byte;
            }
            0x8000...0xffff => {
                panic!("Attempted to write to ROM: {:?}", address);
            }
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::Mapper;
    use crate::Address;

    #[test]
    fn cartridge_is_constructed_from_prg_rom_and_mapper() {
        let prg_rom = Box::new([0u8; 1024]);
        let mapper = Mapper::NROM;
        Cartridge::new(prg_rom, mapper);
    }

    #[test]
    fn nrom_cartridge_maps_0x6000_through_0x7fff_to_ram() {
        let prg_rom = Box::new([0u8; 1024]);
        let mapper = Mapper::NROM;
        let mut cartridge = Cartridge::new(prg_rom, mapper);

        for value in 0x6000..=0x7fff {
            cartridge.write(Address::new(value), value as u8);
            assert_eq!(cartridge.read(Address::new(value)), value as u8);
            assert_eq!(cartridge.prg_ram[value as usize - 0x6000], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x8000_through_0xffff_to_rom() {
        let mut prg_rom = Box::new([0u8; 0x8000]);

        for (i, item) in prg_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        let mapper = Mapper::NROM;
        let cartridge = Cartridge::new(prg_rom, mapper);

        for value in 0x8000..=0xffff {
            assert_eq!(cartridge.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    #[should_panic]
    fn nrom_cartridge_cannot_write_to_read_only_memory() {
        let prg_rom = Box::new([0u8; 0x8000]);

        let mapper = Mapper::NROM;
        let mut cartridge = Cartridge::new(prg_rom, mapper);

        cartridge.write(Address::new(0x9000), 10);
    }
}
