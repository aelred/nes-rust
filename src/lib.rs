mod address;
mod cartridge;
mod cpu;
mod ines;
mod mapper;
mod memory;
mod serialize;

pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
pub use crate::cpu::OpCode;
pub use crate::cpu::CPU;
pub use crate::ines::INes;
pub use crate::ines::INesReadError;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::serialize::SerializeByte;

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
