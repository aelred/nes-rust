pub use i_nes::INes;

use crate::cartridge::mapper::{AnyMapper, Mapper, PRGAddress};
use crate::cartridge::mapper::{CHRAddress, NROM};
use crate::Address;
use crate::Bus;
use std::fmt::{Debug, Formatter};

mod i_nes;
mod mapper;

#[derive(Debug)]
pub struct Cartridge {
    mapper: AnyMapper,
    prg: PRGState,
    chr: CHRState,
}

impl Cartridge {
    fn new(
        prg_rom: Box<[u8]>,
        chr_rom: Box<[u8]>,
        chr_ram_enabled: bool,
        mapper: AnyMapper,
    ) -> Self {
        let prg = PRGState {
            rom: prg_rom,
            ram: [0; 0x2000],
            dirty_ram: false,
        };

        let chr = CHRState {
            chr_rom,
            chr_ram_enabled,
            ppu_ram: [0; 0x800],
        };

        Cartridge { mapper, prg, chr }
    }

    pub fn set_ram(&mut self, ram: &[u8]) {
        self.prg.ram.copy_from_slice(ram);
    }

    pub fn changed_ram(&mut self) -> Option<&[u8]> {
        self.prg.changed_ram()
    }

    #[inline]
    pub fn get_prg_chr(&mut self) -> (PRG<'_>, CHR<'_>) {
        let prg = PRG {
            state: &mut self.prg,
            mapper: &self.mapper,
        };
        let chr = CHR {
            state: &mut self.chr,
            mapper: &self.mapper,
        };
        (prg, chr)
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            mapper: NROM.into(),
            prg: PRGState::default(),
            chr: CHRState::default(),
        }
    }
}

struct PRGState {
    rom: Box<[u8]>,
    ram: [u8; 0x2000],
    dirty_ram: bool,
}

impl PRGState {
    /// Return RAM if it changed since the last call.
    fn changed_ram(&mut self) -> Option<&[u8]> {
        if !self.dirty_ram {
            return None;
        }

        self.dirty_ram = false;
        Some(&self.ram)
    }
}

impl Debug for PRGState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PRG").finish()
    }
}

impl Default for PRGState {
    fn default() -> Self {
        Self {
            rom: Box::new([0; 0x8000]),
            ram: [0; _],
            dirty_ram: false,
        }
    }
}

/// Program memory on a NES cartridge, connected to the CPU
pub struct PRG<'a> {
    state: &'a mut PRGState,
    mapper: &'a AnyMapper,
}

impl Bus for PRG<'_> {
    fn read(&mut self, address: Address) -> u8 {
        match self.mapper.map_cpu(address) {
            PRGAddress::ROM(index) => self.state.rom[index % self.state.rom.len()],
            PRGAddress::RAM(index) => self.state.ram[index],
            PRGAddress::Unmapped => panic!("Out of addressable range: {:?}", address),
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        // Write to a register, if mapper has one at this address
        if self.mapper.write_register(address, byte) {
            return;
        }

        // Otherwise write to RAM
        match self.mapper.map_cpu(address) {
            PRGAddress::ROM(_) => panic!("Writing to PRG-ROM not supported"),
            PRGAddress::RAM(index) => {
                self.state.dirty_ram = true;
                self.state.ram[index] = byte
            }
            PRGAddress::Unmapped => panic!("Out of addressable range: {:?}", address),
        }
    }
}

struct CHRState {
    chr_rom: Box<[u8]>,
    chr_ram_enabled: bool,
    ppu_ram: [u8; 0x800],
}

impl Debug for CHRState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CHR")
            .field("chr_ram_enabled", &self.chr_ram_enabled)
            .finish()
    }
}

impl Default for CHRState {
    fn default() -> Self {
        Self {
            chr_rom: Box::new([0; 0x2000]),
            chr_ram_enabled: false,
            ppu_ram: [0; _],
        }
    }
}

/// Character memory on a NES cartridge, stores pattern tables and is connected to the PPU
pub struct CHR<'a> {
    state: &'a mut CHRState,
    mapper: &'a AnyMapper,
}

impl Bus for CHR<'_> {
    fn read(&mut self, address: Address) -> u8 {
        match self.mapper.map_ppu(address) {
            CHRAddress::ROM(index) => self.state.chr_rom[index],
            CHRAddress::RAM(index) => {
                self.state.ppu_ram[self.mapper.nametable_mirroring().map_index(index)]
            }
            CHRAddress::Unmapped => panic!("Out of addressable range: {:?}", address),
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match self.mapper.map_ppu(address) {
            CHRAddress::ROM(index) => {
                debug_assert!(
                    self.state.chr_ram_enabled,
                    "Attempted to write to CHR-ROM, but writing is not enabled"
                );
                self.state.chr_rom[index] = byte;
            }
            CHRAddress::RAM(index) => {
                self.state.ppu_ram[self.mapper.nametable_mirroring().map_index(index)] = byte;
            }
            CHRAddress::Unmapped => panic!("Out of addressable range: {:?}", address),
        }
    }
}

/// Describes how each of four logical nametables are mapped to one of two physical nametables.
/// 0b0000abcd - where each bit indicates if it maps to the lower or upper nametable.
#[derive(Debug, Copy, Clone)]
struct NametableMirroring(u8);

impl NametableMirroring {
    const UPPER: Self = NametableMirroring(0b0000);
    const LOWER: Self = NametableMirroring(0b1111);
    const HORIZONTAL: Self = NametableMirroring(0b0011);
    const VERTICAL: Self = NametableMirroring(0b0101);

    fn map_index(&self, index: usize) -> usize {
        let logical_nametable = index / 0x400;
        let physical_nametable = self.logical_to_physical_nametable(logical_nametable);

        (physical_nametable * 0x400) + (index % 0x400)
    }

    fn logical_to_physical_nametable(&self, logical_nametable: usize) -> usize {
        ((self.0 & (0b1000 >> logical_nametable)) != 0) as usize
    }
}

impl Default for NametableMirroring {
    fn default() -> Self {
        // TODO: don't have this default, instead use iNES header
        Self::VERTICAL
    }
}

#[cfg(test)]
mod tests {
    use crate::Address;

    use super::*;

    #[test]
    fn cartridge_is_constructed_from_prg_rom_chr_rom_and_mapper() {
        let prg_rom = Box::new([0u8; 1024]);
        let chr_rom = Box::new([0u8; 1024]);
        let mapper = NROM.into();
        Cartridge::new(prg_rom, chr_rom, false, mapper);
    }

    #[test]
    fn rom_cartridge_maps_0x6000_through_0x7fff_to_prg_ram() {
        let mut cartridge = nrom_cartridge();
        let mut prg = PRG {
            state: &mut cartridge.prg,
            mapper: &cartridge.mapper,
        };

        for value in 0x6000..=0x7fff {
            prg.write(Address::new(value), value as u8);
            assert_eq!(prg.read(Address::new(value)), value as u8);
            assert_eq!(prg.state.ram[value as usize - 0x6000], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x8000_through_0xffff_to_prg_rom() {
        let mut cartridge = nrom_cartridge();
        let mut prg = PRG {
            state: &mut cartridge.prg,
            mapper: &cartridge.mapper,
        };

        for (i, item) in prg.state.rom.iter_mut().enumerate() {
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
        let mapper = NROM.into();

        for (i, item) in prg_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        let mut cartridge = Cartridge::new(prg_rom, chr_rom, false, mapper);
        let mut prg = PRG {
            state: &mut cartridge.prg,
            mapper: &cartridge.mapper,
        };

        for value in 0xc000..=0xffff {
            assert_eq!(prg.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x0000_through_0x1fff_to_chr_rom() {
        let mut cartridge = nrom_cartridge();
        let mut chr = CHR {
            state: &mut cartridge.chr,
            mapper: &cartridge.mapper,
        };

        for (i, item) in chr.state.chr_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x0000..=0x1fff {
            assert_eq!(chr.read(Address::new(value)), value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_maps_0x2000_through_0x27ff_to_ppu_ram() {
        let mut cartridge = nrom_cartridge();
        let mut chr = CHR {
            state: &mut cartridge.chr,
            mapper: &cartridge.mapper,
        };

        for (i, item) in chr.state.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x2000..=0x27ff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.state.ppu_ram[value as usize - 0x2000], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_mirrors_0x2800_through_0x2fff_to_ppu_ram() {
        let mut cartridge = nrom_cartridge();
        let mut chr = CHR {
            state: &mut cartridge.chr,
            mapper: &cartridge.mapper,
        };

        for (i, item) in chr.state.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x2800..=0x2fff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.state.ppu_ram[value as usize - 0x2800], value as u8);
        }
    }

    #[test]
    fn nrom_cartridge_mirrors_0x3000_through_0x3eff_to_ppu_ram() {
        let mut cartridge = nrom_cartridge();
        let mut chr = CHR {
            state: &mut cartridge.chr,
            mapper: &cartridge.mapper,
        };

        for (i, item) in chr.state.ppu_ram.iter_mut().enumerate() {
            *item = i as u8;
        }

        for value in 0x3000..=0x37ff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.state.ppu_ram[value as usize - 0x3000], value as u8);
        }

        for value in 0x3800..=0x3eff {
            chr.write(Address::new(value), value as u8);
            assert_eq!(chr.read(Address::new(value)), value as u8);
            assert_eq!(chr.state.ppu_ram[value as usize - 0x3800], value as u8);
        }
    }

    #[test]
    #[should_panic]
    fn nrom_cartridge_cannot_write_to_read_only_memory() {
        let mut cartridge = nrom_cartridge();
        let mut prg = PRG {
            state: &mut cartridge.prg,
            mapper: &cartridge.mapper,
        };

        prg.write(Address::new(0x5000), 10);
    }

    fn nrom_cartridge() -> Cartridge {
        let prg_rom = Box::new([0u8; 0x8000]);
        let chr_rom = Box::new([0u8; 0x8000]);
        let mapper = NROM.into();
        Cartridge::new(prg_rom, chr_rom, false, mapper)
    }
}
