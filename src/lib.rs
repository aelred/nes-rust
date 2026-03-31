#![allow(clippy::upper_case_acronyms)] // Allow upper case acronyms like NES, CPU because I think it's more readable!

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
use anyhow::Result;
use apu::APU;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use wasm_bindgen::prelude::wasm_bindgen;
use web_time::{Duration, Instant};

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
pub mod runtime;

pub const WIDTH: u16 = 256;
pub const HEIGHT: u16 = 240;
pub const NES_FREQ: f64 = 1_789_773.0;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub fn run() {
    if let Err(e) = run_inner() {
        log::error!("Error: {}", e);
    }
}

fn run_inner() -> Result<()> {
    ActiveRuntime::init_log(log::Level::Info)?;
    ActiveRuntime::run()
}

pub trait NESDisplay {
    fn draw_pixel(&mut self, color: Color);
    fn enter_vblank(&mut self);
}

impl NESDisplay for () {
    fn draw_pixel(&mut self, _: Color) {}
    fn enter_vblank(&mut self) {}
}

type Buffer = [u8; WIDTH as usize * HEIGHT as usize * 4];

pub fn display_triple_buffer() -> (FrontBuffer, BackBuffer) {
    let intermediate_buffer = Arc::new(IntermediateBuffer {
        buffer: AtomicPtr::new(Box::into_raw(Box::new([0; _]))),
        dirty: AtomicBool::new(false),
    });

    let front = FrontBuffer {
        front_buffer: Some(Box::new([0; _])),
        intermediate_buffer: intermediate_buffer.clone(),
    };

    let back = BackBuffer {
        back_buffer: Some(Box::new([0; _])),
        x: 0,
        y: 0,
        vblank: false,
        intermediate_buffer,
    };

    (front, back)
}

pub struct FrontBuffer {
    front_buffer: Option<Box<Buffer>>,
    intermediate_buffer: Arc<IntermediateBuffer>,
}

impl FrontBuffer {
    pub fn read_buffer(&mut self) -> &Buffer {
        if self.intermediate_buffer.dirty.swap(false, AcqRel) {
            let old_buffer = self.front_buffer.take().unwrap();
            self.front_buffer = Some(self.intermediate_buffer.swap(old_buffer));
        }

        self.front_buffer.as_ref().unwrap()
    }
}

pub struct BackBuffer {
    back_buffer: Option<Box<Buffer>>,
    x: usize,
    y: usize,
    vblank: bool,
    intermediate_buffer: Arc<IntermediateBuffer>,
}

impl NESDisplay for BackBuffer {
    fn draw_pixel(&mut self, color: Color) {
        self.vblank = false;

        let buffer = self.back_buffer.as_mut().unwrap();

        let offset = (self.y * WIDTH as usize + self.x) * 4;
        if offset + 3 < buffer.len() {
            let (r, g, b) = color.to_rgb();
            buffer[offset] = r;
            buffer[offset + 1] = g;
            buffer[offset + 2] = b;
            buffer[offset + 3] = 0xFF;
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
        if self.vblank {
            return;
        }

        self.x = 0;
        self.y = 0;
        self.vblank = true;

        let old_buffer = self.back_buffer.take().unwrap();
        self.back_buffer = Some(self.intermediate_buffer.swap(old_buffer));
        self.intermediate_buffer.dirty.store(true, Release);
    }
}

struct IntermediateBuffer {
    buffer: AtomicPtr<Buffer>,
    dirty: AtomicBool,
}

impl IntermediateBuffer {
    fn swap(&self, buffer: Box<Buffer>) -> Box<Buffer> {
        let new_ptr = Box::into_raw(buffer);
        let old_ptr = self.buffer.swap(new_ptr, AcqRel);
        // SAFETY: the pointer is always valid and exclusive
        unsafe { Box::from_raw(old_ptr) }
    }
}

impl Drop for IntermediateBuffer {
    fn drop(&mut self) {
        let ptr = self.buffer.load(Acquire);
        // SAFETY: the pointer is always valid and exclusive
        let boxed = unsafe { Box::from_raw(ptr) };
        drop(boxed)
    }
}

pub trait NESSpeaker {
    fn emit(&mut self, wave: f32);
}

impl NESSpeaker for () {
    fn emit(&mut self, _wave: f32) {}
}

#[derive(Debug)]
pub struct NES<D, S> {
    cpu: CPU,
    display: D,
    speaker: S,
}

impl<D: NESDisplay, S: NESSpeaker> NES<D, S> {
    pub fn new(cartridge: Cartridge, display: D, speaker: S) -> Self {
        NES {
            cpu: Self::cpu_from_cartridge(cartridge),
            display,
            speaker,
        }
    }

    pub fn run(&mut self, commands: Receiver<Command>, events: Sender<Event>) -> ! {
        let start = Instant::now();
        let mut cycles: u64 = 0;
        let mut paused = false;

        // Target CPU cycles per loop before sleeping and checking events.
        // Should be small enough to not fill the audio buffer, but too small adds overhead.
        const CYCLES_PER_LOOP: u64 = 5000;

        loop {
            let mut loop_cycles = 0;
            if paused {
                // Don't tick CPU if paused, pretend we ran cycles to sleep a reasonable time.
                loop_cycles = CYCLES_PER_LOOP;
            } else {
                while loop_cycles < CYCLES_PER_LOOP {
                    loop_cycles += self.tick() as u64;
                }
            }
            cycles += loop_cycles;

            let expected_time = Duration::from_secs_f64(cycles as f64 / NES_FREQ);
            let actual_time = start.elapsed();
            if actual_time < expected_time {
                thread::sleep(expected_time - actual_time);
            }

            for command in commands.try_iter() {
                match command {
                    Command::Press(buttons) => self.controller().press(buttons),
                    Command::Release(buttons) => self.controller().release(buttons),
                    Command::LoadRam(ram) => {
                        self.cpu.memory().prg().ram_mut().copy_from_slice(&ram);
                    }
                    Command::LoadCartridge(cartridge) => {
                        self.load_cartridge(cartridge);
                    }
                    Command::Pause => {
                        paused = true;
                    }
                    Command::Resume => {
                        paused = false;
                    }
                }
            }

            if let Some(ram) = self.cpu.memory().prg().changed_ram() {
                // TODO: ideally don't allocate
                let event = Event::RamChanged(Vec::from(ram));
                let _ = events.send(event);
            }
        }
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

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cpu = Self::cpu_from_cartridge(cartridge);
        // Reset display since we'll start drawing from the top-left again
        self.display.enter_vblank();
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
            self.display.draw_pixel(color);
        }

        if output.vblank {
            self.display.enter_vblank();
        }
    }

    fn tick_apu(&mut self) {
        let apu = self.cpu.memory().apu();
        let wave = apu.tick();
        self.speaker.emit(wave);
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

pub enum Command {
    Press(Buttons),
    Release(Buttons),
    LoadRam(Vec<u8>),
    LoadCartridge(Cartridge),
    Pause,
    Resume,
}

pub enum Event {
    RamChanged(Vec<u8>),
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
