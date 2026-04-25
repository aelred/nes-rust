use std::fmt::{Debug, Formatter};

use crate::cpu::Tickable;
use crate::Address;

pub trait Memory: Sized {
    /// This method takes a mutable reference because reading from memory can sometimes trigger
    /// state changes.
    ///
    /// e.g. when reading from the PPU status register, bit 7 of the register is reset.
    fn read(&mut self, address: Address) -> u8;
    fn write(&mut self, address: Address, byte: u8);
}

impl<M: Memory> Memory for &mut M {
    fn read(&mut self, address: Address) -> u8 {
        (**self).read(address)
    }

    fn write(&mut self, address: Address, byte: u8) {
        (**self).write(address, byte);
    }
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

impl Debug for ArrayMemory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayMemory").finish()
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

impl Tickable for ArrayMemory {
    fn tick(&mut self) -> bool {
        false
    }
}
