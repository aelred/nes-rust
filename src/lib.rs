mod address;
mod addressing_modes;
mod cpu;
mod instructions;
mod opcodes;

pub use crate::address::Address;
pub use crate::cpu::CPU;
pub use crate::opcodes::OpCode;

use num_traits::FromPrimitive;

pub trait SerializeBytes {
    const SIZE: u8;

    fn size(&self) -> u8 {
        Self::SIZE
    }

    fn serialize(self, dest: &mut [u8]);

    fn deserialize(source: &[u8]) -> Self;
}

impl SerializeBytes for i8 {
    const SIZE: u8 = 1;

    fn serialize(self, dest: &mut [u8]) {
        dest[0] = self as u8;
    }

    fn deserialize(source: &[u8]) -> Self {
        source[0] as i8
    }
}

impl SerializeBytes for u8 {
    const SIZE: u8 = 1;

    fn serialize(self, dest: &mut [u8]) {
        dest[0] = self;
    }

    fn deserialize(source: &[u8]) -> Self {
        source[0]
    }
}

impl SerializeBytes for OpCode {
    const SIZE: u8 = 1;

    fn serialize(self, dest: &mut [u8]) {
        dest[0] = self as u8;
    }

    fn deserialize(source: &[u8]) -> Self {
        OpCode::from_u8(source[0]).expect("Unrecognised opcode")
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        {
            let mut vec: Vec<u8> = vec![];
            $(
                let mut instr_vec = vec![0; $crate::SerializeBytes::size(&$data) as usize];
                $crate::SerializeBytes::serialize($data, &mut instr_vec);
                vec.extend(instr_vec);
            )*
            vec
        }
    };
}
