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
pub use crate::serialize::SerializeBytes;

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
                    for byte in $crate::SerializeBytes::serialize($data) {
                        $crate::Memory::write(&mut memory, addr, byte);
                        addr += 1u16;
                    }
                )*
            )*
            memory
        }
    };
    ($offset: expr => $data: expr) => {
        mem!{$offset => { $data }}
    };
}
