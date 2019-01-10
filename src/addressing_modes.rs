use crate::cpu::Address;
use crate::cpu::Addressable;

pub trait ValueAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8;
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

impl ValueAddressingMode for FlexibleAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            FlexibleAddressingMode::Immediate => addressable.immediate(),
            FlexibleAddressingMode::ZeroPage => addressable.zero_page(),
            FlexibleAddressingMode::ZeroPageX => addressable.zero_page_x(),
            FlexibleAddressingMode::Absolute => addressable.absolute(),
            FlexibleAddressingMode::AbsoluteX => addressable.absolute_x(),
            FlexibleAddressingMode::AbsoluteY => addressable.absolute_y(),
            FlexibleAddressingMode::IndexedIndirect => addressable.indexed_indirect(),
            FlexibleAddressingMode::IndirectIndexed => addressable.indirect_indexed(),
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

impl ValueAddressingMode for StoreAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            StoreAddressingMode::ZeroPage => addressable.zero_page(),
            StoreAddressingMode::ZeroPageX => addressable.zero_page_x(),
            StoreAddressingMode::Absolute => addressable.absolute(),
            StoreAddressingMode::AbsoluteX => addressable.absolute_x(),
            StoreAddressingMode::AbsoluteY => addressable.absolute_y(),
            StoreAddressingMode::IndexedIndirect => addressable.indexed_indirect(),
            StoreAddressingMode::IndirectIndexed => addressable.indirect_indexed(),
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

impl ValueAddressingMode for ShiftAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            ShiftAddressingMode::Accumulator => addressable.accumulator(),
            ShiftAddressingMode::ZeroPage => addressable.zero_page(),
            ShiftAddressingMode::ZeroPageX => addressable.zero_page_x(),
            ShiftAddressingMode::Absolute => addressable.absolute(),
            ShiftAddressingMode::AbsoluteX => addressable.absolute_x(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BITAddressingMode {
    ZeroPage,
    Absolute,
}

impl ValueAddressingMode for BITAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            BITAddressingMode::ZeroPage => addressable.zero_page(),
            BITAddressingMode::Absolute => addressable.absolute(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CompareAddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
}

impl ValueAddressingMode for CompareAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            CompareAddressingMode::Immediate => addressable.immediate(),
            CompareAddressingMode::ZeroPage => addressable.zero_page(),
            CompareAddressingMode::Absolute => addressable.absolute(),
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

impl ValueAddressingMode for IncDecAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        match self {
            IncDecAddressingMode::ZeroPage => addressable.zero_page(),
            IncDecAddressingMode::ZeroPageX => addressable.zero_page_x(),
            IncDecAddressingMode::Absolute => addressable.absolute(),
            IncDecAddressingMode::AbsoluteX => addressable.absolute_x(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum JumpAddressingMode {
    Absolute,
    Indirect,
}

impl JumpAddressingMode {
    pub fn fetch_address(self, addressable: &mut Addressable) -> Address {
        match self {
            JumpAddressingMode::Absolute => addressable.absolute_address(),
            JumpAddressingMode::Indirect => addressable.indirect_address(),
        }
    }
}

impl ValueAddressingMode for JumpAddressingMode {
    fn fetch(self, addressable: &mut Addressable) -> &mut u8 {
        let address = self.fetch_address(addressable);
        addressable.deref_address(address)
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

#[derive(Debug, Copy, Clone)]
pub enum LDYAddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
}

#[derive(Debug, Copy, Clone)]
pub enum STXAddressingMode {
    ZeroPage,
    ZeroPageY,
    Absolute,
}

#[derive(Debug, Copy, Clone)]
pub enum STYAddressingMode {
    ZeroPage,
    ZeroPageX,
    Absolute,
}
