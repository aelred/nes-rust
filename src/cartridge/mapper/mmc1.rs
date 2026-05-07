use crate::cartridge::mapper::{Mapper, PRGAddress};
use crate::cartridge::NametableMirroring;
use crate::Address;
use std::cell::Cell;

const _16KB: usize = 16_384;

#[derive(Debug, Default)]
pub struct MMC1 {
    last_bank: u8,
    bank: Cell<u8>,
    nametable_mirroring: Cell<NametableMirroring>,
    shift_register: Cell<u8>,
    shift_register_writes: Cell<u8>,
}

impl MMC1 {
    pub fn new(prg_rom: &[u8]) -> Self {
        Self {
            last_bank: ((prg_rom.len() / _16KB) - 1) as u8,
            ..Default::default()
        }
    }
}

impl Mapper for MMC1 {
    fn map(&self, address: Address) -> PRGAddress {
        match address.index() {
            0x6000..=0x7fff => PRGAddress::RAM(address.index() - 0x6000),
            0x8000..=0xbfff => PRGAddress::ROM(bank_index(address - 0x8000, self.bank.get())),
            0xc000..=0xffff => PRGAddress::ROM(bank_index(address - 0xc000, self.last_bank)),
            _ => PRGAddress::Unmapped,
        }
    }

    fn write_register(&self, address: Address, data: u8) -> bool {
        if !matches!(address.index(), 0x8000..=0xffff) {
            return false;
        }

        let reset = (data >> 7) & 1 == 1;
        if reset {
            self.shift_register.set(0);
            self.shift_register_writes.set(0);
            return true;
        }

        self.shift_register.update(|s| s >> 1);
        self.shift_register.update(|s| s | (data & 1) << 4);
        self.shift_register_writes.update(|w| w + 1);

        if self.shift_register_writes.get() < 5 {
            return true;
        }

        let value = self.shift_register.get();
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

                self.nametable_mirroring.set(nametable_mirroring);
            }
            0xa000..=0xbfff => {
                if value != 0 {
                    todo!("Support MMC1 CHR bank 0, value={value}");
                }
            }
            0xc000..=0xdfff => {
                if value != 0 {
                    todo!("Support MMC1 CHR bank 1, value={value}");
                }
            }
            0xe000..=0xffff => {
                self.bank.set(value & 0b1111);
            }
            _ => {
                panic!("Out of addressable range: {:?}", address);
            }
        }

        self.shift_register.set(0);
        self.shift_register_writes.set(0);

        true
    }

    fn nametable_mirroring(&self) -> NametableMirroring {
        self.nametable_mirroring.get()
    }
}

fn bank_index(address: Address, bank: u8) -> usize {
    bank as usize * _16KB + address.index()
}
