use crate::address::Address;
use crate::Memory;

use super::CPU;
use super::Reference;
use super::ReferenceAddressingMode;

macro_rules! def_addressing_modes {
    ($($name:ident { $($mode:ident),* $(,)* })*) => {
        $(
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub enum $name {
            $(
            $mode,
            )*
        }

        impl ReferenceAddressingMode for $name {
            fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
                match self {
                    $(
                    $name::$mode => cpu.exec_addressing_mode(AddressingMode::$mode),
                    )*
                }
            }
        }
        )*
    };
}

def_addressing_modes! {
    FlexibleAddressingMode {
        Immediate,
        ZeroPage,
        ZeroPageX,
        Absolute,
        AbsoluteX,
        AbsoluteY,
        IndexedIndirect,
        IndirectIndexed,
    }

    StoreAddressingMode {
        ZeroPage,
        ZeroPageX,
        Absolute,
        AbsoluteX,
        AbsoluteY,
        IndexedIndirect,
        IndirectIndexed,
    }

    ShiftAddressingMode {
        Accumulator,
        ZeroPage,
        ZeroPageX,
        Absolute,
        AbsoluteX,
    }

    BITAddressingMode {
        ZeroPage,
        Absolute,
    }

    CompareAddressingMode {
        Immediate,
        ZeroPage,
        Absolute,
    }

    IncDecAddressingMode {
        ZeroPage,
        ZeroPageX,
        Absolute,
        AbsoluteX,
    }

    JumpAddressingMode {
        Absolute,
        Indirect,
    }

    LDXAddressingMode {
        Immediate,
        ZeroPage,
        ZeroPageY,
        Absolute,
        AbsoluteY,
    }

    LDYAddressingMode {
        Immediate,
        ZeroPage,
        ZeroPageX,
        Absolute,
        AbsoluteX,
    }

    STXAddressingMode {
        ZeroPage,
        ZeroPageY,
        Absolute,
    }

    STYAddressingMode {
        ZeroPage,
        ZeroPageX,
        Absolute,
    }

    LAXAddressingMode {
        ZeroPage,
        ZeroPageY,
        Absolute,
        AbsoluteY,
        IndexedIndirect,
        IndirectIndexed,
    }
}

impl JumpAddressingMode {
    pub fn fetch_address<M: Memory>(self, cpu: &mut CPU<M>) -> Address {
        match self {
            JumpAddressingMode::Absolute => cpu.absolute_address(),
            JumpAddressingMode::Indirect => cpu.indirect_address(),
        }
    }
}

enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl<M: Memory> CPU<M> {
    fn exec_addressing_mode(&mut self, addressing_mode: AddressingMode) -> Reference {
        match addressing_mode {
            AddressingMode::Accumulator => Reference::Accumulator,
            AddressingMode::Immediate => {
                let reference = Reference::Address(self.program_counter());
                self.fetch_at_program_counter();
                reference
            }
            AddressingMode::ZeroPage => {
                let address = Address::from_bytes(0, self.fetch_at_program_counter());
                Reference::Address(address)
            }
            AddressingMode::ZeroPageX => {
                let operand: u8 = self.fetch_at_program_counter();
                let address = Address::from_bytes(0, operand.wrapping_add(self.x()));
                Reference::Address(address)
            }
            AddressingMode::ZeroPageY => {
                let operand: u8 = self.fetch_at_program_counter();
                let address = Address::from_bytes(0, operand.wrapping_add(self.y()));
                Reference::Address(address)
            }
            AddressingMode::Absolute => {
                let address = self.absolute_address();
                Reference::Address(address)
            },
            AddressingMode::AbsoluteX => {
                let address = self.absolute_address() + u16::from(self.x());
                Reference::Address(address)
            }
            AddressingMode::AbsoluteY => {
                let address = self.absolute_address() + u16::from(self.y());
                Reference::Address(address)
            }
            AddressingMode::Indirect => {
                let address = self.indirect_address();
                Reference::Address(address)
            },
            AddressingMode::IndexedIndirect => {
                let offset = self.fetch_at_program_counter().wrapping_add(self.x());
                let address = self.read_zero_page_address(offset);
                Reference::Address(address)
            }
            AddressingMode::IndirectIndexed => {
                let offset = self.fetch_at_program_counter();
                let address = self.read_zero_page_address(offset) + u16::from(self.y());
                Reference::Address(address)
            }
        }
    }

    fn absolute_address(&mut self) -> Address {
        self.fetch_address_at_program_counter()
    }

    fn indirect_address(&mut self) -> Address {
        let address_of_address = self.fetch_address_at_program_counter();
        self.read_address(address_of_address)
    }

    fn read_zero_page_address(&self, offset: u8) -> Address {
        let lower = self.read(Address::from_bytes(0, offset));
        let higher = self.read(Address::from_bytes(0, offset.wrapping_add(1)));
        Address::from_bytes(higher, lower)
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::*;
    use crate::mem;

    use super::*;
    use super::AddressingMode::*;

    #[test]
    fn immediate_addressing_mode_fetches_given_value() {
        let mut cpu = CPU::with_memory(mem! {56u8});

        let reference = cpu.exec_addressing_mode(Immediate);
        assert_eq!(cpu.read_reference(reference), 56);
    }

    #[test]
    fn accumulator_addressing_mode_fetches_accumulator_value() {
        let mut cpu = CPU::with_memory(mem! {LDA_IMMEDIATE, 76u8});
        cpu.run_instruction();
        assert_eq!(cpu.read_reference(Reference::Accumulator), 76);
    }

    #[test]
    fn zero_page_addressing_mode_fetches_value_at_given_zero_page_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 15u8 }
            15 => { 35u8 }
        ));

        let reference = cpu.exec_addressing_mode(ZeroPage);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn zero_page_x_addressing_mode_fetches_value_at_given_zero_page_address_offset_by_x() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 15u8 }
            18 => { 35u8 }
        ));
        cpu.set_x(3);

        let reference = cpu.exec_addressing_mode(ZeroPageX);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn zero_page_x_addressing_mode_wraps() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0xFFu8 }
        ));
        cpu.set_x(1);

        let reference = cpu.exec_addressing_mode(ZeroPageX);
        assert_eq!(cpu.read_reference(reference), 0xFF);
    }

    #[test]
    fn zero_page_y_addressing_mode_fetches_value_at_given_zero_page_address_offset_by_y() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 15u8 }
            18 => { 35u8 }
        ));
        cpu.set_y(3);

        let reference = cpu.exec_addressing_mode(ZeroPageY);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn zero_page_y_addressing_mode_wraps() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0xFFu8 }
        ));
        cpu.set_y(1);

        let reference = cpu.exec_addressing_mode(ZeroPageY);
        assert_eq!(cpu.read_reference(reference), 0xFF);
    }

    #[test]
    fn absolute_addressing_mode_fetches_values_at_given_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32, 0x04 }
            0x432 => { 35u8 }
        ));

        let reference = cpu.exec_addressing_mode(Absolute);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn absolute_x_addressing_mode_fetches_values_at_given_address_offset_by_x() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32, 0x04 }
            0x435 => { 35u8 }
        ));
        cpu.set_x(3);

        let reference = cpu.exec_addressing_mode(AbsoluteX);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn absolute_y_addressing_mode_fetches_values_at_given_address_offset_by_y() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32, 0x04 }
            0x435 => { 35u8 }
        ));
        cpu.set_y(3);

        let reference = cpu.exec_addressing_mode(AbsoluteY);
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn indirect_addressing_mode_fetches_address_at_given_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32, 0x04 }
            0x432 => { 0x35, 0 }
        ));

        let address = cpu.indirect_address();
        assert_eq!(address, Address::new(0x35));
    }

    #[test]
    fn indirect_addressing_mode_wraps_at_end_of_page() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0xff, 0x04 }
            0x4ff => { 0x34 }
            0x400 => { 0x12 }
        ));

        let address = cpu.indirect_address();
        assert_eq!(address, Address::new(0x1234));
    }

    #[test]
    fn indexed_indirect_addressing_mode_fetches_address_at_given_zero_page_address_offset_by_x() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32 }
            0x35 => { 0x34, 0x12 }
            0x1234 => { 57 }
        ));
        cpu.set_x(3);

        let reference = cpu.exec_addressing_mode(IndexedIndirect);
        assert_eq!(cpu.read_reference(reference), 57);
    }

    #[test]
    fn indexed_indirect_addressing_mode_wraps_on_zero_page_overflow() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32 }
            0x31 => { 0x34, 0x12 }
            0x1234 => { 57 }
        ));
        cpu.set_x(255);

        let reference = cpu.exec_addressing_mode(IndexedIndirect);
        assert_eq!(cpu.read_reference(reference), 57);
    }

    #[test]
    fn indexed_indirect_addressing_mode_wraps_address_read_from_zero_page() {
        let mut cpu = CPU::with_memory(mem!(
            0x00 => { 0xff }
            0xff => { 0x12 }
            0xff12 => { 57 }
        ));
        cpu.set_x(0);

        let reference = cpu.exec_addressing_mode(IndexedIndirect);
        assert_eq!(cpu.read_reference(reference), 57);
    }

    #[test]
    fn indirect_indexed_addressing_mode_fetches_address_offset_by_y_at_given_zero_page_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 0x32 }
            0x32 => { 0x34, 0x12 }
            0x1237 => { 57 }
        ));
        cpu.set_y(3);

        let reference = cpu.exec_addressing_mode(IndirectIndexed);
        assert_eq!(cpu.read_reference(reference), 57);
    }

    #[test]
    fn indirect_indexed_addressing_mode_wraps_address_read_from_zero_page() {
        let mut cpu = CPU::with_memory(mem!(
            0x00 => { 0xff }
            0xff => { 0x12 }
            0xff12 => { 57 }
        ));
        cpu.set_y(0);

        let reference = cpu.exec_addressing_mode(IndirectIndexed);
        assert_eq!(cpu.read_reference(reference), 57);
    }
}
