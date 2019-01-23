use crate::Address;

pub trait Memory: Sized {
    /// This method takes a mutable reference because reading from memory can sometimes trigger
    /// state changes.
    ///
    /// e.g. when reading from the PPU status register, bit 7 of the register is reset.
    fn read(&mut self, address: Address) -> u8;
    fn write(&mut self, address: Address, byte: u8);
}

pub struct ArrayMemory([u8; 0x10000]);

impl ArrayMemory {
    pub fn slice(&self) -> &[u8] {
        &self.0
    }
}

impl Default for ArrayMemory {
    fn default() -> Self {
        ArrayMemory([0; 0x10000])
    }
}

impl Memory for ArrayMemory {
    fn read(&mut self, address: Address) -> u8 {
        self.0[address.index()]
    }

    fn write(&mut self, address: Address, byte: u8) {
        self.0[address.index()] = byte;
    }
}

impl<'a, T: Memory> Memory for &'a mut T {
    fn read(&mut self, address: Address) -> u8 {
        T::read(self, address)
    }

    fn write(&mut self, address: Address, byte: u8) {
        T::write(self, address, byte)
    }
}
