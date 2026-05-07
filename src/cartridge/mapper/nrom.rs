use crate::cartridge::mapper::{Mapper, PRGAddress};
use crate::Address;

#[derive(Debug)]
pub struct NROM;

impl Mapper for NROM {
    fn map(&self, address: Address) -> PRGAddress {
        match address.index() {
            0x6000..=0x7fff => PRGAddress::RAM(address.index() - 0x6000),
            0x8000..=0xffff => PRGAddress::ROM(address.index() - 0x8000),
            _ => PRGAddress::Unmapped,
        }
    }
}
