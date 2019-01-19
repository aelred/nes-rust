use std::cell::RefCell;
use std::rc::Rc;

pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
use crate::cartridge::CHR;
use crate::cartridge::PRG;
pub use crate::cpu::CPU;
pub use crate::cpu::Instruction;
pub use crate::cpu::instructions;
use crate::cpu::NESCPUMemory;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
use crate::memory::NESPPUMemory;
pub use crate::ppu::Color;
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

type StandardPPU<'a> = Rc<RefCell<PPU<NESPPUMemory<&'a mut CHR>>>>;
type StandardCPU<'a> = CPU<NESCPUMemory<&'a mut PRG, StandardPPU<'a>>>;

pub struct NES<'a, D> {
    cpu: StandardCPU<'a>,
    ppu: StandardPPU<'a>,
    display: D,
}

impl<'a, D: NESDisplay> NES<'a, D> {
    pub fn new(cartridge: &'a mut Cartridge, display: D) -> Self {
        let ppu_memory = NESPPUMemory::new(&mut cartridge.chr);
        let ppu = Rc::new(RefCell::new(PPU::with_memory(ppu_memory)));

        let cpu_memory = NESCPUMemory::new(&mut cartridge.prg, Rc::clone(&ppu));
        let cpu = CPU::with_memory(cpu_memory);

        NES { cpu, ppu, display }
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
        self.cpu.run_instruction();
        let color = self.ppu.borrow_mut().tick();
        self.display.draw_pixel(color);
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
