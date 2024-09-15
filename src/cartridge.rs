use std::fmt::{Debug, Formatter};

use crate::mapper::Mapper;
use crate::Address;
use crate::Memory;

pub struct Cartridge {
    pub prg: PRG,
    pub chr: CHR,
}

impl Cartridge {
    pub fn new(
        prg_rom: Box<[u8]>,
        chr_rom: Box<[u8]>,
        chr_ram_enabled: bool,
        mapper: Mapper,
    ) -> Self {
        let prg_rom_window = match mapper {
            Mapper::NROM => 0x4000,
            Mapper::UxROM => 0x4000,
            Mapper::MMC1 => 0x4000,
            Mapper::Namco129 => 0x2000,
            #[allow(unreachable_patterns)] // Allow because we might add more mappers
            _ => unimplemented!("Unsupported mapper {:?}", mapper),
        };

        let prg_rom_len = prg_rom.len() as u16;

        let prg = PRG {
            rom: prg_rom,
            rom_window: prg_rom_window.min(prg_rom_len),
            rom_bank: 0,
            ram: [0; 0x2000],
        };

        let chr = CHR {
            chr_rom,
            chr_ram_enabled,
            ppu_ram: [0; 0x800],
        };

        Cartridge { prg, chr }
    }
}

/// Program memory on a NES cartridge, connected to the CPU
pub struct PRG {
    rom: Box<[u8]>,
    rom_window: u16,
    rom_bank: u8,
    ram: [u8; 0x2000],
}

impl PRG {
    fn read_bank(&self, bank: u8, address: Address) -> u8 {
        let window = self.rom_window as usize;
        self.rom[bank as usize * window + (address.index() % window)]
    }

    fn last_bank(&self) -> u8 {
        ((self.rom.len() / (self.rom_window as usize)) - 1) as u8
    }
}

impl Debug for PRG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PRG").finish()
    }
}

impl Memory for PRG {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x6000..=0x7fff => self.ram[address.index() - 0x6000],
            0x8000..=0xbfff => self.read_bank(self.rom_bank, address - 0x8000),
            0xc000..=0xffff => self.read_bank(self.last_bank(), address - 0xc000),
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x6000..=0x7fff => {
                self.ram[address.index() - 0x6000] = byte;
            }
            0x8000..=0xffff => {
                self.rom_bank = byte;
            }
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

/// Character memory on a NES cartridge, stores pattern tables and is connected to the PPU
pub struct CHR {
    chr_rom: Box<[u8]>,
    chr_ram_enabled: bool,
    ppu_ram: [u8; 0x800],
}

impl Debug for CHR {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CHR")
            .field("chr_ram_enabled", &self.chr_ram_enabled)
            .finish()
    }
}

impl Memory for CHR {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x0000..=0x1fff => self.chr_rom[address.index()],
            0x2000..=0x3eff => self.ppu_ram[(address.index() - 0x2000) % 0x800],
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x0000..=0x1fff => {
                debug_assert!(
                    self.chr_ram_enabled,
                    "Attempted to write to CHR-ROM, but writing is not enabled"
                );
                self.chr_rom[address.index()] = byte
            }
            0x2000..=0x3eff => self.ppu_ram[(address.index() - 0x2000) % 0x800] = byte,
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mapper::Mapper;
    use crate::Address;

    use super::*;

    #[test]
    fn cartridge_is_constructed_from_prg_rom_chr_rom_and_mapper() {
        let prg_rom = Box::new([0u8; 1024]);
        let chr_rom = Box::new([0u8; 1024]);
        let mapper = Mapper::NROM;
        Cartridge::new(prg_rom, chr_rom, false, mapper);
    }

    #[test]
    fn rom_cartridge_maps_0x6000_through_0x7fff_to_prg_ram() {
        let mut prg = nrom_cartridge().prg;

        for value in 0x6000..=0x7fff {
            prg.write(Address::new(value), value as u8);
            assert_eq!(prg.read(Address::new(value)), value as u8);
            assert_eq!(prg.ram[value as usize - 0x6000], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x8000_through_0xffff_to_prg_rom() {
        let mut prg = nrom_cartridge().prg;

        for (i, item) in prg.rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x8000..=0xffff {
            assert_eq!(prg.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_mirrors_rom_if_not_large_enough() {
        let mut prg_rom = Box::new([0u8; 0x4000]);
        let chr_rom = Box::new([0u8; 0x8000]);
        let mapper = Mapper::NROM;

        for (i, item) in prg_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        let mut prg = Cartridge::new(prg_rom, chr_rom, false, mapper).prg;

        for value in 0xc000..=0xffff {
            assert_eq!(prg.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x0000_through_0x1fff_to_chr_rom() {
        let mut chr = nrom_cartridge().chr;

        for (i, item) in chr.chr_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x0000..=0x1fff {
            assert_eq!(chr.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x2000_through_0x27ff_to_ppu_ram() {
        let mut chr = nrom_cartridge().chr;

        for (i, item) in chr.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x2000..=0x27ff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.ppu_ram[value as usize - 0x2000], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_mirrors_0x2800_through_0x2fff_to_ppu_ram() {
        let mut chr = nrom_cartridge().chr;

        for (i, item) in chr.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x2800..=0x2fff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.ppu_ram[value as usize - 0x2800], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_mirrors_0x3000_through_0x3eff_to_ppu_ram() {
        let mut chr = nrom_cartridge().chr;

        for (i, item) in chr.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x3000..=0x37ff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.ppu_ram[value as usize - 0x3000], value as u8);
        }

        for value in 0x3800..=0x3eff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.ppu_ram[value as usize - 0x3800], value as u8);
        }
    }

    #[test]
    #[should_panic]
    fn nrom_cartridge_cannot_write_to_read_only_memory() {
        let mut prg = nrom_cartridge().prg;
        prg.write(Address::new(0x9000), 10);
    }

    fn nrom_cartridge() -> Cartridge {
        let prg_rom = Box::new([0u8; 0x8000]);
        let chr_rom = Box::new([0u8; 0x8000]);
        let mapper = Mapper::NROM;
        Cartridge::new(prg_rom, chr_rom, false, mapper)
    }
}
