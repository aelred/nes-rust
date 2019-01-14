mod address;
mod addressing_modes;
mod cpu;
mod instructions;
mod opcodes;
mod serialize;
mod memory;
mod ines;
mod mapper;
mod cartridge;

pub use crate::address::Address;
pub use crate::memory::Memory;
pub use crate::memory::ArrayMemory;
pub use crate::cpu::CPU;
pub use crate::opcodes::OpCode;
pub use crate::serialize::SerializeByte;
pub use crate::ines::INes;
pub use crate::cartridge::Cartridge;
pub use crate::ines::INesReadError;

use crate::memory::NESMemory;

pub struct NES {
    cpu: CPU<NESMemory<Cartridge>>,
}

impl NES {
    pub fn new(cartridge: Cartridge) -> Self {
        let memory = NESMemory::new(cartridge);
        let cpu = CPU::with_memory(memory);
        NES { cpu }
    }

    pub fn tick(&mut self) {
        self.cpu.run_instruction();
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        mem!{0 => { $($data),* }}
    };
    ($( $offset: expr => { $( $data: expr ),* } )*) => {
        {
            let mut memory = $crate::ArrayMemory::default();
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
