#![allow(clippy::upper_case_acronyms)] // Allow upper case acronyms like NES, CPU because I think it's more readable!

pub use crate::address::Address;
use crate::apu::APU;
use crate::audio::AudioSink;
pub use crate::cartridge::Cartridge;
pub use crate::cpu::instructions;
pub use crate::cpu::CPUState;
pub use crate::cpu::Instruction;
use crate::cpu::NESCPUMemory;
pub use crate::cpu::Tickable;
pub use crate::cpu::CPU;
pub use crate::i_nes::INes;
pub use crate::i_nes::INesReadError;
pub use crate::input::Buttons;
use crate::input::Controller;
pub use crate::memory::ArrayMemory;
pub use crate::memory::Memory;
pub use crate::ppu::Color;
use crate::ppu::{NESPPUMemory, PPUState, RealPPU};
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
    /// TODO: flatten this out (WIP)
    cpu: CPUState,
    ppu: PPUState,
    apu: APU,
    cartridge: Cartridge,
    internal_ram: [u8; 0x800],
    controller: Controller,
    additional_memory: ArrayMemory, // TODO: fake memory to support unsupported memory space
    palette_ram: [u8; 0x20],
    video_out: BackBuffer,
}

impl NES {
    pub fn new(cartridge: Cartridge, video_out: BackBuffer, audio_out: AudioSink) -> Self {
        let mut this = Self {
            cpu: CPUState::default(),
            ppu: PPUState::default(),
            apu: APU::new(audio_out),
            cartridge,
            internal_ram: [0; _],
            controller: Controller::default(),
            additional_memory: ArrayMemory::default(),
            // Initialise whole palette to black
            palette_ram: [0x0F; _],
            video_out,
        };

        let (mut cpu_memory, _) = this.build_cpu();
        this.cpu = CPUState::from_memory(&mut cpu_memory);

        this
    }

    pub fn program_counter(&mut self) -> Address {
        self.cpu.program_counter()
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.cpu.set_program_counter(address);
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        let mut video_out = std::mem::take(&mut self.video_out);
        // Draw from top left
        video_out.reset();

        let audio_out = self.apu.disconnect_audio();

        *self = Self::new(cartridge, video_out, audio_out);
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        let (mut cpu_memory, _) = self.build_cpu();
        cpu_memory.read(address)
    }

    pub fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub fn tick(&mut self) -> u8 {
        let (cpu_memory, cpu_state) = self.build_cpu();
        CPU::new(cpu_memory, cpu_state).run_instruction()
    }

    fn build_cpu(&mut self) -> (NESCPUMemory<'_>, &mut CPUState) {
        let (prg, chr) = self.cartridge.get_prg_chr();

        let ppu_memory = NESPPUMemory::new(&mut self.palette_ram, chr);
        let ppu = RealPPU::new(ppu_memory, &mut self.video_out, &mut self.ppu);

        let cpu_memory = NESCPUMemory::new(
            &mut self.internal_ram,
            prg,
            ppu,
            &mut self.apu,
            &mut self.controller,
            &mut self.additional_memory,
        );
        (cpu_memory, &mut self.cpu)
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
