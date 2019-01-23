pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
use crate::cartridge::CHR;
use crate::cartridge::PRG;
pub use crate::cpu::CPU;
pub use crate::cpu::Instruction;
pub use crate::cpu::instructions;
use crate::cpu::NESCPUMemory;
pub use crate::cpu::RunningCPU;
use crate::cpu::RunningNESCPUMemory;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::ppu::Color;
use crate::ppu::NESPPUMemory;
use crate::ppu::PPU;
use crate::ppu::RunningPPU;
pub use crate::serialize::SerializeByte;

mod address;
mod cartridge;
mod cpu;
mod i_nes;
mod mapper;
mod memory;
mod ppu;
mod serialize;

pub trait NESDisplay {
    fn draw_pixel(&mut self, color: Color);
}

pub struct NoDisplay;

impl NESDisplay for NoDisplay {
    fn draw_pixel(&mut self, _: Color) {}
}

pub struct NES<'a, D> {
    cpu: CPU,
    cpu_memory: NESCPUMemory<&'a mut PRG>,
    ppu: PPU<NESPPUMemory<&'a mut CHR>>,
    display: D,
}

impl<'a, D: NESDisplay> NES<'a, D> {
    pub fn new(cartridge: &'a mut Cartridge, display: D) -> Self {
        let ppu_memory = NESPPUMemory::new(&mut cartridge.chr);
        let mut ppu = PPU::with_memory(ppu_memory);

        let mut cpu_memory = NESCPUMemory::new(&mut cartridge.prg);
        let mut running_cpu_memory = RunningNESCPUMemory::new(&mut cpu_memory, &mut ppu);
        let cpu = CPU::from_memory(&mut running_cpu_memory);

        NES {
            cpu,
            cpu_memory,
            ppu,
            display,
        }
    }

    pub fn program_counter(&mut self) -> Address {
        self.cpu.program_counter()
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.cpu.set_program_counter(address);
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        let mut memory = RunningNESCPUMemory::new(&mut self.cpu_memory, &mut self.ppu);
        memory.read(address)
    }

    pub fn tick(&mut self) {
        self.tick_cpu();
        self.tick_ppu();
    }

    fn tick_cpu(&mut self) {
        let memory = RunningNESCPUMemory::new(&mut self.cpu_memory, &mut self.ppu);
        let mut cpu = RunningCPU::new(&mut self.cpu, memory);
        cpu.run_instruction();
    }

    fn tick_ppu(&mut self) {
        let mut ppu = RunningPPU::new(&mut self.ppu, &mut self.cpu);
        for _ in 0..10 {
            if let Some(color) = ppu.tick() {
                self.display.draw_pixel(color);
            }
        }
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),*) => {
        mem!{0 => { $($data),* }}
    };
    ($( $offset: expr => { $( $data: expr ),* } )*) => {
        {
            #[allow(unused_variables, unused_mut)]
            let mut memory = $crate::ArrayMemory::default();
            $(
                #[allow(unused_variables, unused_mut)]
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
