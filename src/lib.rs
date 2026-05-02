#![allow(clippy::upper_case_acronyms)] // Allow upper case acronyms like NES, CPU because I think it's more readable!

pub use crate::address::Address;
pub use crate::bus::ArrayMemory;
pub use crate::bus::Bus;
pub use crate::cartridge::Cartridge;
pub use crate::cartridge::INes;
pub use crate::cpu::instructions;
pub use crate::cpu::CPUState;
pub use crate::cpu::Instruction;
pub use crate::cpu::Tickable;
pub use crate::cpu::CPU;
pub use crate::input::Buttons;
pub use crate::nes::NES;
pub use crate::ppu::Color;
mod address;
mod apu;
pub mod audio;
mod bus;
mod cartridge;
mod cpu;
mod input;
mod nes;
mod ppu;
mod runner;
pub mod runtime;
pub mod video;

pub const WIDTH: u16 = 256;
pub const HEIGHT: u16 = 240;
pub const NES_FREQ: f64 = 1_789_773.0;

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
                    $crate::Bus::write(&mut memory, addr, byte);
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
