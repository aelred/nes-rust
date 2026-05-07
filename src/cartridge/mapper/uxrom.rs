use crate::cartridge::mapper::{Mapper, PRGAddress};
use crate::Address;
use std::cell::Cell;

const _16KB: usize = 16_384;

#[derive(Debug, Default)]
pub struct UxROM {
    last_bank: u8,
    bank: Cell<u8>,
}

impl UxROM {
    pub fn new(prg_rom: &[u8]) -> Self {
        Self {
            last_bank: ((prg_rom.len() / _16KB) - 1) as u8,
            bank: Cell::new(0),
        }
    }
}

impl Mapper for UxROM {
    fn map_cpu(&self, address: Address) -> PRGAddress {
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

        self.bank.set(data);
        true
    }
}

fn bank_index(address: Address, bank: u8) -> usize {
    bank as usize * _16KB + address.index()
}
