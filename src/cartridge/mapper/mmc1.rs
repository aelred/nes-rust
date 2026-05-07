use crate::cartridge::mapper::{CHRAddress, Mapper, PRGAddress};
use crate::cartridge::NametableMirroring;
use crate::Address;
use std::cell::Cell;

const _4KB: usize = 4096;
const _16KB: usize = 16_384;

#[derive(Debug, Default)]
pub struct MMC1 {
    last_prg_bank: u8,
    prg_bank: Cell<u8>,
    chr_bank_4kb_mode: Cell<bool>,
    chr_banks: [Cell<u8>; 2],
    nametable_mirroring: Cell<NametableMirroring>,
    shift_register: Cell<u8>,
    shift_register_writes: Cell<u8>,
}

impl MMC1 {
    pub fn new(prg_rom: &[u8]) -> Self {
        Self {
            last_prg_bank: ((prg_rom.len() / _16KB) - 1) as u8,
            ..Default::default()
        }
    }
}

impl Mapper for MMC1 {
    fn map_cpu(&self, address: Address) -> PRGAddress {
        match address.index() {
            0x6000..=0x7fff => PRGAddress::RAM(address.index() - 0x6000),
            0x8000..=0xbfff => {
                PRGAddress::ROM(prg_bank_index(address - 0x8000, self.prg_bank.get()))
            }
            0xc000..=0xffff => {
                PRGAddress::ROM(prg_bank_index(address - 0xc000, self.last_prg_bank))
            }
            _ => PRGAddress::Unmapped,
        }
    }

    fn map_ppu(&self, address: Address) -> CHRAddress {
        match address.index() {
            0x0000..=0x0fff => {
                let bank = if self.chr_bank_4kb_mode.get() {
                    self.chr_banks[0].get()
                } else {
                    self.chr_banks[0].get() & 0b11110
                };
                CHRAddress::ROM(chr_bank_index(address, bank))
            }
            0x1000..=0x1fff => {
                let bank = if self.chr_bank_4kb_mode.get() {
                    self.chr_banks[1].get()
                } else {
                    (self.chr_banks[0].get() & 0b11110) + 1
                };
                CHRAddress::ROM(chr_bank_index(address - 0x1000, bank))
            }
            0x2000..=0x3eff => CHRAddress::RAM(address.index() - 0x2000),
            _ => CHRAddress::Unmapped,
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

        match address.index() {
            0x8000..=0x9fff => {
                let nametable_mirroring = match value & 0b11 {
                    0b00 => NametableMirroring::LOWER,
                    0b01 => NametableMirroring::UPPER,
                    0b10 => NametableMirroring::VERTICAL,
                    0b11 => NametableMirroring::HORIZONTAL,
                    _ => unreachable!(),
                };
                self.nametable_mirroring.set(nametable_mirroring);

                let prg_bank_mode = (value & 0b1100) >> 2;
                if prg_bank_mode != 3 {
                    log::error!("Unsupported MMC1 PRG bank modes: {prg_bank_mode}")
                }

                let chr_bank_4kb_mode = (value & 0b1_0000) != 0;
                self.chr_bank_4kb_mode.set(chr_bank_4kb_mode);
            }
            0xa000..=0xbfff => {
                let mask = if self.chr_bank_4kb_mode.get() {
                    0b11111
                } else {
                    0b11110
                };
                self.chr_banks[0].set(value & mask);
            }
            0xc000..=0xdfff => {
                if self.chr_bank_4kb_mode.get() {
                    self.chr_banks[1].set(value & 0b11111)
                }
            }
            0xe000..=0xffff => {
                self.prg_bank.set(value & 0b1111);
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

fn prg_bank_index(address: Address, bank: u8) -> usize {
    bank as usize * _16KB + address.index()
}

fn chr_bank_index(address: Address, bank: u8) -> usize {
    bank as usize * _4KB + address.index()
}
