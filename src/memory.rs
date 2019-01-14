use crate::Address;

pub trait Memory: Sized {
    fn read(&self, address: Address) -> u8;
    fn write(&mut self, address: Address, byte: u8);
}

pub type ArrayMemory = [u8; 0x10000];

impl Memory for ArrayMemory {
    fn read(&self, address: Address) -> u8 {
        self[address.index()]
    }

    fn write(&mut self, address: Address, byte: u8) {
        self[address.index()] = byte;
    }
}
