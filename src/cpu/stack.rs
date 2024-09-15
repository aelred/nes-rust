use crate::{Address, Memory};

use super::CPU;

pub const BASE: Address = Address::new(0x0100);

/// S - 8-bit stack pointer.
/// Index into the stack when combined with [BASE].
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct StackPointer(pub u8);

impl StackPointer {
    fn address(&self) -> Address {
        BASE + u16::from(self.0)
    }

    fn decrement(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

impl Default for StackPointer {
    fn default() -> Self {
        Self(0xFF)
    }
}

impl<M: Memory> CPU<M> {
    pub fn push_stack(&mut self, byte: u8) {
        self.write(self.stack_pointer.address(), byte);
        self.stack_pointer.decrement();
    }

    pub fn increment_stack(&mut self) {
        self.stack_pointer.increment();
        self.cycle_count += 1;
    }

    pub fn pull_and_increment_stack(&mut self) -> u8 {
        let stack_address = self.stack_pointer.address();
        self.stack_pointer.increment();
        self.read(stack_address)
    }

    pub fn pull_stack(&mut self) -> u8 {
        self.read(self.stack_pointer.address())
    }
}
