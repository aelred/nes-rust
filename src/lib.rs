pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
use crate::cartridge::CHR;
use crate::cartridge::PRG;
pub use crate::cpu::instructions;
pub use crate::cpu::Instruction;
use crate::cpu::NESCPUMemory;
pub use crate::cpu::CPU;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::ppu::Color;
use crate::ppu::NESPPUMemory;
use crate::ppu::PPU;
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

type StandardPPU<'a> = PPU<NESPPUMemory<&'a mut CHR>>;

pub struct NES<'a, D> {
    cpu: CPU<NESCPUMemory<&'a mut PRG, StandardPPU<'a>>>,
    display: D,
}

impl<'a, D: NESDisplay> NES<'a, D> {
    pub fn new(cartridge: &'a mut Cartridge, display: D) -> Self {
        let ppu_memory = NESPPUMemory::new(&mut cartridge.chr);
        let ppu = PPU::with_memory(ppu_memory);

        let cpu_memory = NESCPUMemory::new(&mut cartridge.prg, ppu);
        let cpu = CPU::from_memory(cpu_memory);

        NES { cpu, display }
    }

    pub fn program_counter(&mut self) -> Address {
        self.cpu.program_counter()
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.cpu.set_program_counter(address);
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        self.cpu.read(address)
    }

    pub fn tick(&mut self) {
        self.tick_cpu();
        self.tick_ppu();
    }

    fn tick_cpu(&mut self) {
        self.cpu.run_instruction();
    }

    fn ppu(&mut self) -> &mut StandardPPU<'a> {
        self.cpu.memory().ppu_registers()
    }

    fn tick_ppu(&mut self) {
        for _ in 0..10 {
            let output = self.ppu().tick();

            if output.interrupt {
                self.cpu.non_maskable_interrupt();
            }

            if let Some(color) = output.color {
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
