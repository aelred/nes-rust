use crate::address::Address;
use crate::cpu::Memory;
use crate::cpu::Reference;
use crate::cpu::CPU;

pub trait ReferenceAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference;
}

pub trait ValueAddressingMode {
    fn fetch<M: Memory>(self, cpu: &mut CPU<M>) -> u8;
}

impl<T: ReferenceAddressingMode> ValueAddressingMode for T {
    fn fetch<M: Memory>(self, cpu: &mut CPU<M>) -> u8 {
        let reference = self.fetch_ref(cpu);
        cpu.read_reference(reference)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FlexibleAddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
}

impl ReferenceAddressingMode for FlexibleAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            FlexibleAddressingMode::Immediate => cpu.immediate(),
            FlexibleAddressingMode::ZeroPage => cpu.zero_page(),
            FlexibleAddressingMode::ZeroPageX => cpu.zero_page_x(),
            FlexibleAddressingMode::Absolute => cpu.absolute(),
            FlexibleAddressingMode::AbsoluteX => cpu.absolute_x(),
            FlexibleAddressingMode::AbsoluteY => cpu.absolute_y(),
            FlexibleAddressingMode::IndexedIndirect => cpu.indexed_indirect(),
            FlexibleAddressingMode::IndirectIndexed => cpu.indirect_indexed(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StoreAddressingMode {
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
}

impl ReferenceAddressingMode for StoreAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            StoreAddressingMode::ZeroPage => cpu.zero_page(),
            StoreAddressingMode::ZeroPageX => cpu.zero_page_x(),
            StoreAddressingMode::Absolute => cpu.absolute(),
            StoreAddressingMode::AbsoluteX => cpu.absolute_x(),
            StoreAddressingMode::AbsoluteY => cpu.absolute_y(),
            StoreAddressingMode::IndexedIndirect => cpu.indexed_indirect(),
            StoreAddressingMode::IndirectIndexed => cpu.indirect_indexed(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ShiftAddressingMode {
    Accumulator,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
}

impl ReferenceAddressingMode for ShiftAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            ShiftAddressingMode::Accumulator => Reference::Accumulator,
            ShiftAddressingMode::ZeroPage => cpu.zero_page(),
            ShiftAddressingMode::ZeroPageX => cpu.zero_page_x(),
            ShiftAddressingMode::Absolute => cpu.absolute(),
            ShiftAddressingMode::AbsoluteX => cpu.absolute_x(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BITAddressingMode {
    ZeroPage,
    Absolute,
}

impl ReferenceAddressingMode for BITAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            BITAddressingMode::ZeroPage => cpu.zero_page(),
            BITAddressingMode::Absolute => cpu.absolute(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CompareAddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
}

impl ReferenceAddressingMode for CompareAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            CompareAddressingMode::Immediate => cpu.immediate(),
            CompareAddressingMode::ZeroPage => cpu.zero_page(),
            CompareAddressingMode::Absolute => cpu.absolute(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IncDecAddressingMode {
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
}

impl ReferenceAddressingMode for IncDecAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            IncDecAddressingMode::ZeroPage => cpu.zero_page(),
            IncDecAddressingMode::ZeroPageX => cpu.zero_page_x(),
            IncDecAddressingMode::Absolute => cpu.absolute(),
            IncDecAddressingMode::AbsoluteX => cpu.absolute_x(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum JumpAddressingMode {
    Absolute,
    Indirect,
}

impl JumpAddressingMode {
    pub fn fetch_address<M: Memory>(self, cpu: &mut CPU<M>) -> Address {
        match self {
            JumpAddressingMode::Absolute => cpu.absolute_address(),
            JumpAddressingMode::Indirect => cpu.indirect_address(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LDXAddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageY,
    Absolute,
    AbsoluteY,
}

impl ReferenceAddressingMode for LDXAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            LDXAddressingMode::Immediate => cpu.immediate(),
            LDXAddressingMode::ZeroPage => cpu.zero_page(),
            LDXAddressingMode::ZeroPageY => cpu.zero_page_y(),
            LDXAddressingMode::Absolute => cpu.absolute(),
            LDXAddressingMode::AbsoluteY => cpu.absolute_y(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LDYAddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
}

impl ReferenceAddressingMode for LDYAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            LDYAddressingMode::Immediate => cpu.immediate(),
            LDYAddressingMode::ZeroPage => cpu.zero_page(),
            LDYAddressingMode::ZeroPageX => cpu.zero_page_x(),
            LDYAddressingMode::Absolute => cpu.absolute(),
            LDYAddressingMode::AbsoluteX => cpu.absolute_x(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum STXAddressingMode {
    ZeroPage,
    ZeroPageY,
    Absolute,
}

impl ReferenceAddressingMode for STXAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            STXAddressingMode::ZeroPage => cpu.zero_page(),
            STXAddressingMode::ZeroPageY => cpu.zero_page_y(),
            STXAddressingMode::Absolute => cpu.absolute(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum STYAddressingMode {
    ZeroPage,
    ZeroPageX,
    Absolute,
}

impl ReferenceAddressingMode for STYAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference {
        match self {
            STYAddressingMode::ZeroPage => cpu.zero_page(),
            STYAddressingMode::ZeroPageX => cpu.zero_page_x(),
            STYAddressingMode::Absolute => cpu.absolute(),
        }
    }
}

impl<M: Memory> CPU<M> {
    fn immediate(&mut self) -> Reference {
        let reference = Reference::Address(self.program_counter());
        self.fetch_at_program_counter::<u8>();
        reference
    }

    fn zero_page(&mut self) -> Reference {
        Reference::Address(Address::zero_page(self.fetch_at_program_counter()))
    }

    fn zero_page_x(&mut self) -> Reference {
        unimplemented!();
    }

    fn zero_page_y(&mut self) -> Reference {
        unimplemented!();
    }

    fn absolute(&mut self) -> Reference {
        Reference::Address(self.absolute_address())
    }

    fn absolute_address(&mut self) -> Address {
        self.fetch_at_program_counter()
    }

    fn absolute_x(&mut self) -> Reference {
        Reference::Address(self.absolute_address() + self.x())
    }

    fn absolute_y(&mut self) -> Reference {
        Reference::Address(self.absolute_address() + self.y())
    }

    fn indirect_address(&mut self) -> Address {
        unimplemented!();
    }

    fn indirect_indexed(&mut self) -> Reference {
        unimplemented!();
    }

    fn indexed_indirect(&mut self) -> Reference {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem;
    use crate::opcodes::OpCode::*;

    #[test]
    fn immediate_addressing_mode_fetches_given_value() {
        let mut cpu = CPU::with_memory(mem! {56u8});

        let reference = cpu.immediate();
        assert_eq!(cpu.read_reference(reference), 56);
    }

    #[test]
    fn accumulator_addressing_mode_fetches_accumulator_value() {
        let mut cpu = CPU::with_memory(mem! {LDAImmediate, 76u8});
        cpu.run_instruction();
        assert_eq!(cpu.read_reference(Reference::Accumulator), 76);
    }

    #[test]
    fn zero_page_addressing_mode_fetches_value_at_given_zero_page_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { 15u8 }
            15 => { 35u8 }
        ));

        let reference = cpu.zero_page();
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn absolute_addressing_mode_fetches_values_at_given_address() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { Address::new(432) }
            432 => { 35u8 }
        ));

        let reference = cpu.absolute();
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn absolute_x_addressing_mode_fetches_values_at_given_address_offset_by_x() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { Address::new(432) }
            435 => { 35u8 }
        ));
        cpu.set_x(3);

        let reference = cpu.absolute_x();
        assert_eq!(cpu.read_reference(reference), 35);
    }

    #[test]
    fn absolute_y_addressing_mode_fetches_values_at_given_address_offset_by_y() {
        let mut cpu = CPU::with_memory(mem!(
            0 => { Address::new(432) }
            435 => { 35u8 }
        ));
        cpu.set_y(3);

        let reference = cpu.absolute_y();
        assert_eq!(cpu.read_reference(reference), 35);
    }

}
