use crate::address::Address;
use crate::cpu::Addressable;
use crate::cpu::Memory;
use crate::cpu::Reference;

pub trait ReferenceAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference;
}

pub trait ValueAddressingMode {
    fn fetch<M: Memory>(self, addressable: &mut Addressable<M>) -> u8;
}

impl<T: ReferenceAddressingMode> ValueAddressingMode for T {
    fn fetch<M: Memory>(self, addressable: &mut Addressable<M>) -> u8 {
        let reference = self.fetch_ref(addressable);
        addressable.read(reference)
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
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
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

impl ReferenceAddressingMode for StoreAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
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

impl ReferenceAddressingMode for ShiftAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
        match self {
            ShiftAddressingMode::Accumulator => Reference::Accumulator,
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

impl ReferenceAddressingMode for BITAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
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

impl ReferenceAddressingMode for CompareAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
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

impl ReferenceAddressingMode for IncDecAddressingMode {
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
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
    pub fn fetch_address<M: Memory>(self, addressable: &mut Addressable<M>) -> Address {
        match self {
            JumpAddressingMode::Absolute => addressable.absolute_address(),
            JumpAddressingMode::Indirect => addressable.indirect_address(),
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
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
        match self {
            LDXAddressingMode::Immediate => addressable.immediate(),
            LDXAddressingMode::ZeroPage => addressable.zero_page(),
            LDXAddressingMode::ZeroPageY => addressable.zero_page_y(),
            LDXAddressingMode::Absolute => addressable.absolute(),
            LDXAddressingMode::AbsoluteY => addressable.absolute_y(),
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
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
        match self {
            LDYAddressingMode::Immediate => addressable.immediate(),
            LDYAddressingMode::ZeroPage => addressable.zero_page(),
            LDYAddressingMode::ZeroPageX => addressable.zero_page_x(),
            LDYAddressingMode::Absolute => addressable.absolute(),
            LDYAddressingMode::AbsoluteX => addressable.absolute_x(),
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
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
        match self {
            STXAddressingMode::ZeroPage => addressable.zero_page(),
            STXAddressingMode::ZeroPageY => addressable.zero_page_y(),
            STXAddressingMode::Absolute => addressable.absolute(),
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
    fn fetch_ref<M: Memory>(self, addressable: &mut Addressable<M>) -> Reference {
        match self {
            STYAddressingMode::ZeroPage => addressable.zero_page(),
            STYAddressingMode::ZeroPageX => addressable.zero_page_x(),
            STYAddressingMode::Absolute => addressable.absolute(),
        }
    }
}
