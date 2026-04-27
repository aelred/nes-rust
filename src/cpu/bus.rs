use crate::apu::APU;
use crate::cpu::Tickable;
use crate::input::{Controller, Input};
use crate::ppu::{self, RealPPU};
use crate::Bus;
use crate::{cartridge, Address};

pub struct CPUBus<'a, PRG = cartridge::PRG<'a>, PPU = RealPPU<'a>, IN = Controller> {
    internal_ram: &'a mut [u8; 0x800],
    prg: PRG,
    ppu: PPU,
    apu: APU<'a>,
    input: &'a mut IN,
}

impl<'a, PRG: Bus, PPU: ppu::PPU, IN: Input> CPUBus<'a, PRG, PPU, IN> {
    #[inline]
    pub fn new(
        internal_ram: &'a mut [u8; 0x800],
        prg: PRG,
        ppu: PPU,
        apu: APU<'a>,
        input: &'a mut IN,
    ) -> Self {
        Self {
            internal_ram,
            prg,
            ppu,
            apu,
            input,
        }
    }

    fn write_oam_data(&mut self, page: u8) {
        let address = Address::from_bytes(page, 0);

        let mut data = [0; 256];

        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self.read(address + offset as u16);
        }

        self.ppu.write_oam_dma(data);
    }

    fn mirrored_address(address: Address) -> Address {
        match address.index() {
            0x2000..=0x3fff => Address::new(0x2000) + address.bytes() % 8,
            _ => address,
        }
    }
}

impl<PRG: Bus, PPU: ppu::PPU, IN: Input> Bus for CPUBus<'_, PRG, PPU, IN> {
    fn read(&mut self, address: Address) -> u8 {
        match Self::mirrored_address(address).bytes() {
            0x0000..=0x1fff => self.internal_ram[address.index() % 0x0800],
            0x2000 => 0, // open bus
            0x2001 => 0, // open bus
            0x2002 => self.ppu.read_status(),
            0x2003 => 0, // open bus
            0x2004 => self.ppu.read_oam_data(),
            0x2005 => 0, // open bus
            0x2006 => 0, // open bus
            0x2007 => self.ppu.read_data(),
            0x2008..=0x3fff => unreachable!(), // handled by earlier address mirroring
            0x4000..=0x4014 => 0,              // unused
            0x4015 => self.apu.read_status(),
            0x4016 => self.input.read(),
            0x4017 => 0, // TODO: Joystick 2
            0x4018..=0x401f => unimplemented!("APU test functionality"),
            0x4020..=0xffff => self.prg.read(address),
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        match Self::mirrored_address(address).bytes() {
            0x0000..=0x1fff => {
                self.internal_ram[address.index() % 0x0800] = byte;
            }
            0x2000 => self.ppu.write_control(byte),
            0x2001 => self.ppu.write_mask(byte),
            0x2002 => {} // TODO: unsupported, but has some odd open bus behaviour
            0x2003 => self.ppu.write_oam_address(byte),
            0x2004 => self.ppu.write_oam_data(byte),
            0x2005 => self.ppu.write_scroll(byte),
            0x2006 => self.ppu.write_address(byte),
            0x2007 => self.ppu.write_data(byte),
            0x2008..=0x3fff => unreachable!(), // handled by earlier address mirroring
            0x4000 => self.apu.write_pulse_1_flags(byte),
            0x4001 => {} // TODO: pulse 1 sweep
            0x4002 => self.apu.write_pulse_1_timer(byte),
            0x4003 => self.apu.write_pulse_1_length(byte),
            0x4004 => self.apu.write_pulse_2_flags(byte),
            0x4005 => {} // TODO: pulse 2 sweep
            0x4006 => self.apu.write_pulse_2_timer(byte),
            0x4007 => self.apu.write_pulse_2_length(byte),
            0x4008 => self.apu.write_triangle_flags(byte),
            0x4009 => {} // unused
            0x400a => self.apu.write_triangle_timer(byte),
            0x400b => self.apu.write_triangle_length(byte),
            0x400c => self.apu.write_noise_flags(byte),
            0x400d => {} // unused
            0x400e => self.apu.write_noise_mode(byte),
            0x400f => self.apu.write_noise_length(byte),
            0x4010 => {} // TODO: DMC flags
            0x4011 => {} // TODO: DMC direct load
            0x4012 => {} // TODO: DMC sample address
            0x4013 => {} // TODO: DMC sample length
            0x4014 => self.write_oam_data(byte),
            0x4015 => self.apu.write_status(byte),
            0x4016 => self.input.write(byte),
            0x4017 => self.apu.write_frame_counter(byte),
            0x4018..=0x401f => unimplemented!("APU test functionality"),
            0x4020..=0xffff => self.prg.write(address, byte),
        }
    }
}

impl<PRG: Bus, PPU: ppu::PPU, IN: Input> Tickable for CPUBus<'_, PRG, PPU, IN> {
    fn tick(&mut self) -> bool {
        let interrupt = self.ppu.tick();
        self.apu.tick();
        interrupt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apu::APUState;
    use crate::audio::AudioSink;
    use crate::ppu::PPU;
    use crate::ArrayMemory;

    #[test]
    fn can_read_and_write_internal_ram_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();

        for value in 0x0..=0x07ff {
            let address = Address::new(value);

            bus.write(address, value as u8);
            assert_eq!(bus.read(address), value as u8);
        }
    }

    #[test]
    fn cpu_bus_addresses_0x800_to_0x1fff_mirror_internal_ram() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();

        for value in 0x0800..=0x1fff {
            let address = Address::new(value);
            let true_address = Address::new(value % 0x0800);

            bus.write(address, value as u8);
            assert_eq!(bus.read(address), value as u8);
            assert_eq!(bus.read(true_address), value as u8);

            bus.write(true_address, (value + 1) as u8);
            assert_eq!(bus.read(address), (value + 1) as u8);
        }
    }

    #[test]
    fn can_write_ppuctrl_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2000), 0x43);
        assert_eq!(bus.ppu.control, 0x43);
        bus.write(Address::new(0x3ff8), 0x44);
        assert_eq!(bus.ppu.control, 0x44);
    }

    #[test]
    fn can_write_ppumask_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2001), 0x43);
        assert_eq!(bus.ppu.mask, 0x43);
        bus.write(Address::new(0x3ff9), 0x44);
        assert_eq!(bus.ppu.mask, 0x44);
    }

    #[test]
    fn can_read_ppustatus_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.ppu.status = 0x43;
        assert_eq!(bus.read(Address::new(0x2002)), 0x43);
        assert_eq!(bus.read(Address::new(0x3ffa)), 0x43);
    }

    #[test]
    fn can_write_oamaddr_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2003), 0x43);
        assert_eq!(bus.ppu.oam_address, 0x43);
        bus.write(Address::new(0x3ffb), 0x44);
        assert_eq!(bus.ppu.oam_address, 0x44);
    }

    #[test]
    fn can_read_oamdata_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.ppu.oam_data = 0x43;
        assert_eq!(bus.read(Address::new(0x3ffc)), 0x43);
        assert_eq!(bus.read(Address::new(0x2004)), 0x43);
    }

    #[test]
    fn can_write_oamdata_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2004), 0x43);
        assert_eq!(bus.ppu.oam_data, 0x43);
        bus.write(Address::new(0x3ffc), 0x44);
        assert_eq!(bus.ppu.oam_data, 0x44);
    }

    #[test]
    fn can_write_oamdma_in_cpu_bus() {
        let mut expected = [0u8; 256];
        #[allow(clippy::needless_range_loop)] // Cleaner like this
        for i in 0..=255 {
            expected[i] = i as u8;
        }

        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        for i in 0x0200..=0x02ff {
            bus.write(Address::new(i), expected[i as usize % 256]);
        }
        bus.write(Address::new(0x4014), 0x02);

        assert_eq!(bus.ppu.oam_dma, expected);
    }

    #[test]
    fn can_write_ppuscroll_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2005), 0x43);
        assert_eq!(bus.ppu.scroll, 0x43);
        bus.write(Address::new(0x3ffd), 0x44);
        assert_eq!(bus.ppu.scroll, 0x44);
    }

    #[test]
    fn can_write_ppuaddr_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2006), 0x43);
        assert_eq!(bus.ppu.address, 0x43);
        bus.write(Address::new(0x3ffe), 0x44);
        assert_eq!(bus.ppu.address, 0x44);
    }

    #[test]
    fn can_read_ppudata_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x2007), 0x43);
        assert_eq!(bus.ppu.data, 0x43);
        bus.write(Address::new(0x3fff), 0x44);
        assert_eq!(bus.ppu.data, 0x44);
    }

    #[test]
    fn can_write_ppudata_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.ppu.data = 0x43;
        assert_eq!(bus.read(Address::new(0x3fff)), 0x43);
        assert_eq!(bus.read(Address::new(0x2007)), 0x43);
    }

    #[test]
    fn can_read_and_write_cartridge_space_in_cpu_bus() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();

        for value in 0x4020..=0xffff {
            let address = Address::new(value);

            bus.write(address, value as u8);
            assert_eq!(bus.read(address), value as u8);
            assert_eq!(bus.prg.read(address), value as u8);
        }
    }

    #[test]
    fn reading_from_4016_reads_from_input_device() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.input.0 = 24;
        assert_eq!(bus.read(Address::new(0x4016)), 24);
    }

    #[test]
    fn writing_to_4016_writes_to_input_device() {
        let mut bus = TestCPUBus::default();
        let mut bus = bus.bus();
        bus.write(Address::new(0x4016), 52);
        assert_eq!(bus.input.0, 52);
    }

    struct MockPPU {
        control: u8,
        mask: u8,
        status: u8,
        oam_address: u8,
        oam_data: u8,
        scroll: u8,
        address: u8,
        data: u8,
        oam_dma: [u8; 256],
    }

    impl Tickable for MockPPU {
        fn tick(&mut self) -> bool {
            false
        }
    }

    impl PPU for &mut MockPPU {
        fn write_control(&mut self, byte: u8) {
            self.control = byte;
        }

        fn write_mask(&mut self, byte: u8) {
            self.mask = byte;
        }

        fn read_status(&mut self) -> u8 {
            self.status
        }

        fn write_oam_address(&mut self, byte: u8) {
            self.oam_address = byte;
        }

        fn read_oam_data(&mut self) -> u8 {
            self.oam_data
        }

        fn write_oam_data(&mut self, byte: u8) {
            self.oam_data = byte;
        }

        fn write_scroll(&mut self, byte: u8) {
            self.scroll = byte;
        }

        fn write_address(&mut self, byte: u8) {
            self.address = byte;
        }

        fn read_data(&mut self) -> u8 {
            self.data
        }

        fn write_data(&mut self, byte: u8) {
            self.data = byte;
        }

        fn write_oam_dma(&mut self, bytes: [u8; 256]) {
            self.oam_dma = bytes;
        }
    }

    struct MockInput(u8);

    impl Input for MockInput {
        fn read(&mut self) -> u8 {
            self.0
        }

        fn write(&mut self, value: u8) {
            self.0 = value;
        }
    }

    struct TestCPUBus {
        internal_ram: [u8; 0x800],
        prg: ArrayMemory,
        ppu: MockPPU,
        apu: APUState,
        audio_sink: AudioSink,
        input: MockInput,
    }

    impl TestCPUBus {
        fn bus(&mut self) -> CPUBus<'_, &mut ArrayMemory, &mut MockPPU, MockInput> {
            let apu = APU::new(&mut self.audio_sink, &mut self.apu);
            CPUBus::new(
                &mut self.internal_ram,
                &mut self.prg,
                &mut self.ppu,
                apu,
                &mut self.input,
            )
        }
    }

    impl Default for TestCPUBus {
        fn default() -> Self {
            Self {
                internal_ram: [0; _],
                prg: ArrayMemory::default(),
                ppu: MockPPU {
                    control: 0,
                    mask: 0,
                    status: 0,
                    oam_address: 0,
                    oam_data: 0,
                    scroll: 0,
                    address: 0,
                    data: 0,
                    oam_dma: [0; 256],
                },
                apu: APUState::default(),
                audio_sink: AudioSink::default(),
                input: MockInput(0),
            }
        }
    }
}
