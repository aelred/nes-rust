use crate::OpCode;

pub trait SerializeByte {
    fn to_byte(self) -> u8;
}

impl SerializeByte for u8 {
    fn to_byte(self) -> u8 {
        self
    }
}

impl SerializeByte for OpCode {
    fn to_byte(self) -> u8 {
        self as u8
    }
}
