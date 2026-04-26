use crate::mapper::Mapper;
use crate::Address;
use crate::Bus;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};

#[derive(Default, Debug)]
pub struct Cartridge {
    prg: PRGState,
    chr: CHRState,
    config: Configuration,
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

        let config = Configuration::default();

        let prg = PRGState {
            rom: prg_rom,
            bank_mapping: vec![0, last_bank].into(),
            bank_size: prg_bank_size,
            bank_switcher,
            ram: [0; 0x2000],
            dirty_ram: false,
        };

        let chr = CHRState {
            chr_rom,
            chr_ram_enabled,
            ppu_ram: [0; 0x800],
        };

        log::info!(
            "Creating cartridge with PRG ROM of size {} and window of size {}",
            prg_rom_len,
            prg_bank_size
        );

        Cartridge { prg, chr, config }
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
            config: &self.config,
        };
        let chr = CHR {
            state: &mut self.chr,
            config: &self.config,
        };
        (prg, chr)
    }
}

struct PRGState {
    rom: Box<[u8]>,
    bank_mapping: Box<[u8]>,
    bank_size: u16,
    bank_switcher: BankSwitcher,
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
            bank_mapping: Box::new([0]),
            bank_size: 0x8000,
            bank_switcher: BankSwitcher::First,
            ram: [0; _],
            dirty_ram: false,
        }
    }
}

/// Program memory on a NES cartridge, connected to the CPU
pub struct PRG<'a> {
    state: &'a mut PRGState,
    config: &'a Configuration,
}

impl Bus for PRG<'_> {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x6000..=0x7fff => self.state.ram[address.index() - 0x6000],
            0x8000..=0xffff => {
                let relative_address = address - 0x8000;
                let bank_index = relative_address.bytes() / self.state.bank_size;
                let bank = self.state.bank_mapping[bank_index as usize];
                let bank_start = bank_index * self.state.bank_size;
                let bank_address = relative_address - bank_start;
                let bank_size = self.state.bank_size as usize;
                self.state.rom[bank as usize * bank_size + (bank_address.index() % bank_size)]
            }
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x6000..=0x7fff => {
                self.state.dirty_ram = true;
                self.state.ram[address.index() - 0x6000] = byte;
            }
            0x8000..=0xffff => match &mut self.state.bank_switcher {
                BankSwitcher::First => {
                    self.state.bank_mapping[0] = byte;
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
                        return;
                    }

                    *shift_register >>= 1;
                    *shift_register |= (byte & 1) << 4;
                    *writes += 1;

                    if *writes < 5 {
                        return;
                    }

                    let value = *shift_register;
                    // TODO: support other MMC1 registers
                    match address.index() {
                        0x8000..=0x9fff => {
                            // TODO: also support other control flags
                            let nametable_mirroring = match value & 0b11 {
                                0b00 => NametableMirroring::LOWER,
                                0b01 => NametableMirroring::UPPER,
                                0b10 => NametableMirroring::VERTICAL,
                                0b11 => NametableMirroring::HORIZONTAL,
                                _ => unreachable!(),
                            };

                            self.config.nametable_mirroring.set(nametable_mirroring);
                        }
                        0xa000..=0xbfff => {
                            if value != 0 {
                                todo!("Support MMC1 CHR bank 0");
                            }
                        }
                        0xc000..=0xdfff => {
                            todo!("Support MMC1 CHR bank 1");
                        }
                        0xe000..=0xffff => {
                            self.state.bank_mapping[0] = value & 0b1111;
                        }
                        _ => {
                            panic!("Out of addressable range: {:?}", address);
                        }
                    }

                    *shift_register = 0;
                    *writes = 0;
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
    config: &'a Configuration,
}

impl<'a> CHR<'a> {
    fn map_nametable(&self, address: Address) -> usize {
        debug_assert!(0x2000 <= address.index() && address.index() <= 0x3eff);

        let mirroring = self.config.nametable_mirroring.get();

        let index = address.index() - 0x2000;
        let logical_nametable = index / 0x400;
        let physical_nametable = mirroring.logical_to_physical_nametable(logical_nametable);

        (physical_nametable * 0x400) + (index % 0x400)
    }
}

impl Bus for CHR<'_> {
    fn read(&mut self, address: Address) -> u8 {
        match address.index() {
            0x0000..=0x1fff => self.state.chr_rom[address.index()],
            0x2000..=0x3eff => self.state.ppu_ram[self.map_nametable(address)],
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match address.index() {
            0x0000..=0x1fff => {
                debug_assert!(
                    self.state.chr_ram_enabled,
                    "Attempted to write to CHR-ROM, but writing is not enabled"
                );
                self.state.chr_rom[address.index()] = byte
            }
            0x2000..=0x3eff => self.state.ppu_ram[self.map_nametable(address)] = byte,
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }
    }
}

/// Configuration shared between the CHR and PRG.
#[derive(Debug, Default, Clone)]
struct Configuration {
    nametable_mirroring: Cell<NametableMirroring>,
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
        let mut cartridge = nrom_cartridge();
        let mut prg = PRG {
            state: &mut cartridge.prg,
            config: &cartridge.config,
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
            config: &cartridge.config,
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
        let mapper = Mapper::NROM;

        for (i, item) in prg_rom.iter_mut().enumerate() {
            *item = i as u8;
        }

        let mut cartridge = Cartridge::new(prg_rom, chr_rom, false, mapper);
        let mut prg = PRG {
            state: &mut cartridge.prg,
            config: &cartridge.config,
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
            config: &cartridge.config,
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
            config: &cartridge.config,
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
            config: &cartridge.config,
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
            config: &cartridge.config,
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
            config: &cartridge.config,
        };

        prg.write(Address::new(0x5000), 10);
    }

    fn nrom_cartridge() -> Cartridge {
        let prg_rom = Box::new([0u8; 0x8000]);
        let chr_rom = Box::new([0u8; 0x8000]);
        let mapper = Mapper::NROM;
        Cartridge::new(prg_rom, chr_rom, false, mapper)
    }
}
