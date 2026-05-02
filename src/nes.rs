use crate::address::Address;
use crate::apu::{APUState, APU};
use crate::audio::AudioSink;
use crate::cartridge::Cartridge;
use crate::cpu::CPUBus;
use crate::cpu::{CPUState, CPU};
use crate::input::Controller;
use crate::ppu::PPUBus;
use crate::ppu::{PPUState, RealPPU};
use crate::video::BackBuffer;
use crate::Bus;

#[derive(Debug)]
pub struct NES {
    cpu: CPUState,
    ppu: PPUState,
    apu: APUState,
    cartridge: Cartridge,
    controller: Controller,
    palette_ram: [u8; 0x20],
    internal_ram: [u8; 0x800],
    video_out: BackBuffer,
    audio_out: AudioSink,
}

impl NES {
    pub fn new(cartridge: Cartridge, video_out: BackBuffer, audio_out: AudioSink) -> Self {
        let mut this = Self {
            cpu: CPUState::default(),
            ppu: PPUState::default(),
            apu: APUState::default(),
            cartridge,
            controller: Controller::default(),
            // Initialise whole palette to black
            palette_ram: [0x0F; _],
            internal_ram: [0; _],
            video_out,
            audio_out,
        };

        let (mut cpu_bus, _) = this.build_cpu();
        this.cpu = CPUState::from_bus(&mut cpu_bus);

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

        let audio_out = std::mem::take(&mut self.audio_out);

        *self = Self::new(cartridge, video_out, audio_out);
    }

    pub fn read_cpu(&mut self, address: Address) -> u8 {
        let (mut cpu_bus, _) = self.build_cpu();
        cpu_bus.read(address)
    }

    pub fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub fn tick(&mut self) -> u8 {
        let (cpu_bus, cpu_state) = self.build_cpu();
        CPU::new(cpu_bus, cpu_state).run_instruction()
    }

    pub fn changed_ram(&mut self) -> Option<&[u8]> {
        self.cartridge.changed_ram()
    }

    #[inline]
    fn build_cpu(&mut self) -> (CPUBus<'_>, &mut CPUState) {
        let (prg, chr) = self.cartridge.get_prg_chr();

        let ppu_bus = PPUBus::new(&mut self.palette_ram, chr);
        let ppu = RealPPU::new(ppu_bus, &mut self.video_out, &mut self.ppu);

        let apu = APU::new(&mut self.audio_out, &mut self.apu);

        let cpu_bus = CPUBus::new(&mut self.internal_ram, prg, ppu, apu, &mut self.controller);
        (cpu_bus, &mut self.cpu)
    }
}
