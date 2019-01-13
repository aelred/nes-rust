mod address;
mod addressing_modes;
mod cpu;
mod instructions;
mod opcodes;
mod serialize;

pub use crate::address::Address;
pub use crate::cpu::Memory;
pub use crate::cpu::CPU;
pub use crate::opcodes::OpCode;
pub use crate::serialize::SerializeByte;

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        mem!{0 => { $($data),* }}
    };
    ($( $offset: expr => { $( $data: expr ),* } )*) => {
        {
            let mut memory = [0; 0x10000];
            $(
                let mut addr: Address = Address::from($offset);
                $(
                    let byte = $crate::SerializeByte::to_byte($data);
                    $crate::Memory::write(&mut memory, addr, byte);
                    addr += 1u16;
                )*
            )*
            memory
        }
    };
    ($offset: expr => $data: expr) => {
        mem!{$offset => { $data }}
    };
}
