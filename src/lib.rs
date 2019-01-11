mod address;
mod addressing_modes;
mod cpu;
mod instructions;
mod opcodes;

pub use crate::address::Address;
pub use crate::cpu::CPU;
pub use crate::opcodes::OpCode;

pub trait SerializeBytes {
    fn bytes(self) -> Vec<u8>;
}

impl SerializeBytes for i8 {
    fn bytes(self) -> Vec<u8> {
        vec![self as u8]
    }
}

impl SerializeBytes for u8 {
    fn bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl SerializeBytes for OpCode {
    fn bytes(self) -> Vec<u8> {
        vec![self as u8]
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        {
            let mut vec: Vec<u8> = vec![];
            $(vec.extend($crate::SerializeBytes::bytes($data));)*
            vec
        }
    };
}
