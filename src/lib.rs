#![allow(clippy::upper_case_acronyms)] // Allow upper case acronyms like NES, CPU because I think it's more readable!

use std::fmt::{Debug, Formatter};

pub use crate::address::Address;
pub use crate::cartridge::Cartridge;
pub use crate::cpu::instructions;
pub use crate::cpu::Instruction;
use crate::cpu::NESCPUMemory;
pub use crate::cpu::CPU;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::input::Buttons;
use crate::input::Controller;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::ppu::Color;
use crate::ppu::NESPPUMemory;
use crate::ppu::PPU;
pub use crate::runtime::ActiveRuntime;
pub use crate::runtime::Runtime;
pub use crate::serialize::SerializeByte;

mod address;
mod cartridge;
mod cpu;
mod i_nes;
mod input;
mod mapper;
mod memory;
mod ppu;
mod runtime;
mod serialize;

pub const WIDTH: u16 = 256;
pub const HEIGHT: u16 = 240;

#[cfg_attr(feature = "web", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    if let Err(e) = ActiveRuntime::run() {
        log::error!("Error: {}", e);
    }
}

pub trait NESDisplay {
    fn draw_pixel(&mut self, color: Color);
    fn enter_vblank(&mut self);
}

#[derive(Debug)]
pub struct NoDisplay;

impl NESDisplay for NoDisplay {
    fn draw_pixel(&mut self, _: Color) {}
    fn enter_vblank(&mut self) {}
}

pub struct BufferDisplay {
    buffer: Box<[u8; WIDTH as usize * HEIGHT as usize * 4]>,
    x: usize,
    y: usize,
    vblank: bool,
}

impl Default for BufferDisplay {
    fn default() -> Self {
        BufferDisplay {
            buffer: Box::new([0; WIDTH as usize * HEIGHT as usize * 4]),
            x: usize::from(WIDTH) - 8,
            y: usize::from(HEIGHT) - 1,
            vblank: false,
        }
    }
}

impl BufferDisplay {
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    pub fn vblank(&self) -> bool {
        self.vblank
    }
}

impl Debug for BufferDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferDisplay").finish()
    }
}

impl NESDisplay for BufferDisplay {
    fn draw_pixel(&mut self, color: Color) {
        // if self.vblank {
        //     log::debug!("Exit vblank");
        // }
        self.vblank = false;

        let offset = (self.y * WIDTH as usize + self.x) * 4;
        if offset + 3 < self.buffer.len() {
            let (r, g, b) = color.to_rgb();
            self.buffer[offset] = r;
            self.buffer[offset + 1] = g;
            self.buffer[offset + 2] = b;
            self.buffer[offset + 3] = 0xFF;
        }

        self.x += 1;
        if self.x == usize::from(WIDTH) {
            self.x = 0;
            self.y += 1;
            if self.y == usize::from(HEIGHT) {
                self.y = 0;
            }
        }
    }

    fn enter_vblank(&mut self) {
        self.vblank = true;
    }
}

#[derive(Debug)]
pub struct NES<D> {
    cpu: CPU,
    display: D,
}

impl<D: NESDisplay> NES<D> {
    pub fn new(cartridge: Cartridge, display: D) -> Self {
        let ppu_memory = NESPPUMemory::new(cartridge.chr);
        let ppu = PPU::with_memory(ppu_memory);
        let controller = Controller::default();

        let cpu_memory = NESCPUMemory::new(cartridge.prg, ppu, controller);
        let cpu = CPU::from_memory(cpu_memory);

        NES { cpu, display }
    }

    pub fn display(&self) -> &D {
        &self.display
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

    pub fn controller(&mut self) -> &mut Controller {
        self.cpu.memory().input()
    }

    pub fn tick(&mut self) {
        let cpu_cycles = self.tick_cpu();

        // There are 3 PPU cycles to 1 CPU cycle
        for _ in 0..3 * cpu_cycles {
            self.tick_ppu();
        }
    }

    fn tick_cpu(&mut self) -> u8 {
        self.cpu.run_instruction()
    }

    fn ppu(&mut self) -> &mut PPU {
        self.cpu.memory().ppu_registers()
    }

    fn tick_ppu(&mut self) {
        let output = self.ppu().tick();

        if output.interrupt {
            self.cpu.non_maskable_interrupt();
        }

        if let Some(color) = output.color {
            self.display.draw_pixel(color);
        }

        if output.vblank {
            self.display.enter_vblank();
        }
    }
}

#[macro_export]
macro_rules! mem {
    ($( $data: expr ),* $(,)? ) => {
        mem!{0 => { $($data),* }}
    };
    ($( $offset: expr => { $( $data: expr ),* $(,)? } )*) => {
        {
            #[allow(unused_variables, unused_mut)]
            let mut memory = $crate::ArrayMemory::default();
            $(
                #[allow(unused_variables, unused_mut)]
                let mut addr: $crate::Address = $crate::Address::from($offset);
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
