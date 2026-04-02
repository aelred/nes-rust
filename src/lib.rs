#![allow(clippy::upper_case_acronyms)] // Allow upper case acronyms like NES, CPU because I think it's more readable!

pub use crate::address::Address;
use crate::audio::AudioSink;
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
use apu::APU;
use std::fmt::Debug;
use video::BackBuffer;

mod address;
mod apu;
pub mod audio;
mod cartridge;
mod cpu;
mod i_nes;
mod input;
mod mapper;
mod memory;
mod ppu;
mod runner;
pub mod runtime;
pub mod video;

pub const WIDTH: u16 = 256;
pub const HEIGHT: u16 = 240;
pub const NES_FREQ: f64 = 1_789_773.0;

#[derive(Debug)]
pub struct NES {
    cpu: CPU,
    video_out: BackBuffer,
    audio_out: AudioSink,
}

impl NES {
    pub fn new(cartridge: Cartridge, video_out: BackBuffer, audio_out: AudioSink) -> Self {
        NES {
            cpu: Self::cpu_from_cartridge(cartridge),
            video_out,
            audio_out,
        }
    }

    pub fn program_counter(&mut self) -> Address {
        self.cpu.program_counter()
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.cpu.set_program_counter(address);
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cpu = Self::cpu_from_cartridge(cartridge);
        // Draw from top left
        self.video_out.reset();
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        self.cpu.read(address)
    }

    pub fn controller(&mut self) -> &mut Controller {
        self.cpu.memory().input()
    }

    pub fn tick(&mut self) -> u8 {
        let cpu_cycles = self.cpu.run_instruction();

        // There are 3 PPU cycles to 1 CPU cycle
        for _ in 0..3 * cpu_cycles {
            self.tick_ppu();
        }

        for _ in 0..cpu_cycles {
            self.tick_apu();
        }

        cpu_cycles
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
            self.video_out.write(color);
        }
    }

    fn tick_apu(&mut self) {
        let apu = self.cpu.memory().apu();
        let wave = apu.tick();
        self.audio_out.write(wave);
    }

    fn cpu_from_cartridge(cartridge: Cartridge) -> CPU {
        let ppu_memory = NESPPUMemory::new(cartridge.chr);
        let ppu = PPU::with_memory(ppu_memory);
        let controller = Controller::default();
        let apu = APU::default();

        let cpu_memory = NESCPUMemory::new(cartridge.prg, ppu, apu, controller);
        CPU::from_memory(cpu_memory)
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
                    let byte = u8::from($data);
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
