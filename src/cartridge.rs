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
        let prg_bank_size = match mapper {
            Mapper::NROM => 0x4000,
            Mapper::UxROM => 0x4000,
            Mapper::MMC1 => 0x4000,
            Mapper::Namco129 => 0x2000,
            #[allow(unreachable_patterns)] // Allow because we might add more mappers
            _ => unimplemented!("Unsupported mapper {:?}", mapper),
        };

        let prg_rom_len = prg_rom.len();
        let prg_bank_size = prg_bank_size.min(prg_rom_len.try_into().unwrap_or(u16::MAX));
        let last_bank = (prg_rom_len / (prg_bank_size as usize) - 1) as u8;

        let bank_switcher = match mapper {
            Mapper::MMC1 => BankSwitcher::MMC1 {
                shift_register: 0,
                writes: 0,
            },
            _ => BankSwitcher::First,
        };

        let prg = PRG {
            rom: prg_rom,
            bank_mapping: vec![0, last_bank].into(),
            bank_size: prg_bank_size,
            bank_switcher,
            ram: [0; 0x2000],
        };

        let chr = CHR {
            chr_rom,
            chr_ram_enabled,
            ppu_ram: [0; 0x800],
        };

        log::info!(
            "Creating cartridge with PRG ROM of size {} and window of size {}",
            prg_rom_len,
            prg_bank_size
        );

        Cartridge { prg, chr }
    }
}

/// Program memory on a NES cartridge, connected to the CPU
pub struct PRG {
    rom: Box<[u8]>,
    bank_mapping: Box<[u8]>,
    bank_size: u16,
    bank_switcher: BankSwitcher,
    ram: [u8; 0x2000],
}

impl PRG {
    pub fn ram(&mut self) -> &mut [u8] {
        &mut self.ram
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
            0x8000..=0xffff => {
                let relative_address = address - 0x8000;
                let bank_index = relative_address.bytes() / self.bank_size;
                let bank = self.bank_mapping[bank_index as usize];
                let bank_start = bank_index * self.bank_size;
                let bank_address = relative_address - bank_start;
                let bank_size = self.bank_size as usize;
                self.rom[bank as usize * bank_size + (bank_address.index() % bank_size)]
            }
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
            0x8000..=0xffff => match &mut self.bank_switcher {
                BankSwitcher::First => {
                    self.bank_mapping[0] = byte;
                }
                // MMC1 mapper uses a serial interface, where bits are shifted into a shift register.
                // After 5 writes, the shift register is used to update a register.
                BankSwitcher::MMC1 {
                    shift_register,
                    writes,
                } => {
                    let reset = (byte >> 7) & 1 == 1;
                    if reset {
                        *shift_register = 0;
                        *writes = 0;
                    } else {
                        *shift_register >>= 1;
                        *shift_register |= (byte & 1) << 4;
                        *writes += 1;
                        if *writes == 5 {
                            // TODO: support other MMC1 registers
                            match address.index() {
                                0x8000..=0x9fff => {
                                    // TODO: support MMC1 control
                                }
                                0xa000..=0xbfff => {
                                    if *shift_register != 0 {
                                        todo!("Support MMC1 CHR bank 0");
                                    }
                                }
                                0xc000..=0xdfff => {
                                    todo!("Support MMC1 CHR bank 1");
                                }
                                0xe000..=0xffff => {
                                    self.bank_mapping[0] = *shift_register & 0b1111;
                                }
                                _ => {
                                    panic!("Out of addressable range: {:?}", address);
                                }
                            }

                            *shift_register = 0;
                            *writes = 0;
                        }
                    }
                }
            },
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

enum BankSwitcher {
    First,
    MMC1 { shift_register: u8, writes: u8 },
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
        prg.write(Address::new(0x5000), 10);
    }

    fn nrom_cartridge() -> Cartridge {
        let prg_rom = Box::new([0u8; 0x8000]);
        let chr_rom = Box::new([0u8; 0x8000]);
        let mapper = Mapper::NROM;
        Cartridge::new(prg_rom, chr_rom, false, mapper)
    }
}
