mod address;
mod cartridge;
mod cpu;
mod i_nes;
mod mapper;
mod memory;
mod ppu;
mod serialize;

pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
pub use crate::cpu::OpCode;
pub use crate::cpu::CPU;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::serialize::SerializeByte;

use crate::cartridge::PRG;
use crate::memory::NESCPUMemory;
use crate::memory::NESPPUMemory;
use crate::cartridge::CHR;
use crate::ppu::PPU;

pub struct NES<'a> {
    cpu: CPU<NESCPUMemory<&'a mut PRG>>,
    ppu: PPU<NESPPUMemory<&'a mut CHR>>,
}

impl<'a> NES<'a> {
    pub fn new(cartridge: &'a mut Cartridge) -> Self {
        let cpu_memory = NESCPUMemory::new(&mut cartridge.prg);
        let cpu = CPU::with_memory(cpu_memory);
        let ppu_memory = NESPPUMemory::new(&mut cartridge.chr);
        let ppu = PPU::with_memory(ppu_memory);
        NES { cpu, ppu }
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.cpu.set_program_counter(address);
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        self.cpu.read(address)
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
