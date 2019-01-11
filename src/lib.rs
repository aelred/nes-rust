mod address;
mod addressing_modes;
mod cpu;
mod instructions;
mod opcodes;

pub use crate::address::Address;
pub use crate::cpu::CPU;
pub use crate::opcodes::OpCode;

use num_traits::FromPrimitive;

pub trait SerializeBytes: Sized {
    const SIZE: u8;

    type Iter: DoubleEndedIterator<Item = u8>;

    fn serialize(self) -> Self::Iter;

    fn deserialize(source: impl Iterator<Item = u8>) -> Self;
}

pub trait SerializeByte {
    fn to_byte(&self) -> u8;
    fn from_byte(byte: u8) -> Self;
}

impl<T: SerializeByte> SerializeBytes for T {
    const SIZE: u8 = 1;

    type Iter = std::iter::Once<u8>;

    fn serialize(self) -> Self::Iter {
        std::iter::once(self.to_byte())
    }

    fn deserialize(mut source: impl Iterator<Item = u8>) -> Self {
        Self::from_byte(source.next().unwrap())
    }
}

impl SerializeByte for u8 {
    fn to_byte(&self) -> u8 {
        *self
    }

    fn from_byte(byte: u8) -> Self {
        byte
    }
}

impl SerializeByte for i8 {
    fn to_byte(&self) -> u8 {
        *self as u8
    }

    fn from_byte(byte: u8) -> Self {
        byte as i8
    }
}

impl SerializeByte for OpCode {
    fn to_byte(&self) -> u8 {
        *self as u8
    }

    fn from_byte(byte: u8) -> Self {
        OpCode::from_u8(byte).expect("Unrecognised opcode")
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        {
            let mut vec: Vec<u8> = vec![];
            $(
                for byte in $crate::SerializeBytes::serialize($data) {
                    vec.push(byte);
                }
            )*
            vec
        }
    };
}
