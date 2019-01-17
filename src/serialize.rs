use crate::cpu::Instruction;

pub trait SerializeByte {
    fn to_byte(self) -> u8;
}

impl SerializeByte for u8 {
    fn to_byte(self) -> u8 {
        self
    }
}

impl SerializeByte for Instruction {
    fn to_byte(self) -> u8 {
        self.to_opcode()
    }
}
