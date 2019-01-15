mod address;
mod cartridge;
mod cpu;
mod i_nes;
mod mapper;
mod memory;
mod serialize;
mod ppu;

pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
pub use crate::cpu::OpCode;
pub use crate::cpu::CPU;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
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
