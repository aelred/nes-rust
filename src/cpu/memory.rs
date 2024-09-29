use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};

use log::trace;

use crate::apu::APU;
use crate::input::{Controller, Input};
use crate::ppu::{self, PPURegisters};
use crate::ArrayMemory;
use crate::Memory;
use crate::{cartridge, Address};

const PPU_SPACE: Address = Address::new(0x2000);
const PPU_CONTROL: Address = Address::new(0x2000);
const PPU_MASK: Address = Address::new(0x2001);
const PPU_STATUS: Address = Address::new(0x2002);
const OAM_ADDRESS: Address = Address::new(0x2003);
const OAM_DATA: Address = Address::new(0x2004);
const PPU_SCROLL: Address = Address::new(0x2005);
const PPU_ADDRESS: Address = Address::new(0x2006);
const PPU_DATA: Address = Address::new(0x2007);
const APU_SPACE: Address = Address::new(0x4000);
const APU_PULSE_1_FLAGS: Address = Address::new(0x4000);
const APU_PULSE_1_SWEEP: Address = Address::new(0x4001);
const APU_PULSE_1_TIMER: Address = Address::new(0x4002);
const APU_PULSE_1_LENGTH: Address = Address::new(0x4003);
const APU_PULSE_2_FLAGS: Address = Address::new(0x4004);
const APU_PULSE_2_SWEEP: Address = Address::new(0x4005);
const APU_PULSE_2_TIMER: Address = Address::new(0x4006);
const APU_PULSE_2_LENGTH: Address = Address::new(0x4007);
const APU_TRIANGLE_FLAGS: Address = Address::new(0x4008);
const APU_TRIANGLE_TIMER: Address = Address::new(0x400a);
const APU_TRIANGLE_LENGTH: Address = Address::new(0x400b);
const APU_NOISE_FLAGS: Address = Address::new(0x400c);
const APU_NOISE_MODE: Address = Address::new(0x400e);
const APU_NOISE_LENGTH: Address = Address::new(0x400f);
const APU_DMC_FLAGS: Address = Address::new(0x4010);
const APU_DMC_DIRECT_LOAD: Address = Address::new(0x4011);
const APU_DMC_SAMPLE_ADDRESS: Address = Address::new(0x4012);
const APU_DMC_SAMPLE_LENGTH: Address = Address::new(0x4013);
const OAM_DMA: Address = Address::new(0x4014);
const APU_STATUS: Address = Address::new(0x4015);
const JOY1_ADDRESS: Address = Address::new(0x4016);
const APU_FRAME_COUNTER: Address = Address::new(0x4017);
const PRG_SPACE: Address = Address::new(0x4020);

pub struct NESCPUMemory<PRG = cartridge::PRG, PPU = ppu::PPU, IN = Controller> {
    internal_ram: [u8; 0x800],
    prg: PRG,
    ppu_registers: PPU,
    apu: APU,
    input: IN,
    the_rest: ArrayMemory, // TODO
}

impl<PRG: Memory, PPU: PPURegisters, IN: Input> NESCPUMemory<PRG, PPU, IN> {
    pub fn new(prg: PRG, ppu_registers: PPU, apu: APU, input: IN) -> Self {
        NESCPUMemory {
            internal_ram: [0; 0x800],
            prg,
            ppu_registers,
            apu,
            input,
            the_rest: ArrayMemory::default(),
        }
    }

    pub fn ppu_registers(&mut self) -> &mut PPU {
        &mut self.ppu_registers
    }

    pub fn apu(&mut self) -> &mut APU {
        &mut self.apu
    }

    pub fn input(&mut self) -> &mut IN {
        &mut self.input
    }

    pub fn prg(&mut self) -> &mut PRG {
        &mut self.prg
    }

    fn write_oam_data(&mut self, page: u8) {
        let address = Address::from_bytes(page, 0);

        let mut data = [0; 256];

        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self.read(address + offset as u16);
        }

        self.ppu_registers.write_oam_dma(data);
    }
}

impl<PRG: Debug, PPU: Debug, IN: Debug> Debug for NESCPUMemory<PRG, PPU, IN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NESCPUMemory")
            .field("prg", &self.prg)
            .field("ppu_registers", &self.ppu_registers)
            .field("input", &self.input)
            .field("the_rest", &self.the_rest)
            .finish()
    }
}

impl<PRG: Memory, PPU: PPURegisters, IN: Input> Memory for NESCPUMemory<PRG, PPU, IN> {
    fn read(&mut self, address: Address) -> u8 {
        if address >= PRG_SPACE {
            self.prg.read(address)
        } else if address == JOY1_ADDRESS {
            self.input.read()
        } else if address == APU_STATUS {
            self.apu.read_status()
        } else if address >= APU_SPACE {
            self.the_rest.read(address) // TODO
        } else if address >= PPU_SPACE {
            let mirrored = PPU_SPACE + (address.index() % 8) as u16;
            let ppu_registers = self.ppu_registers.borrow_mut();
            match mirrored {
                PPU_STATUS => ppu_registers.read_status(),
                OAM_DATA => ppu_registers.read_oam_data(),
                PPU_DATA => ppu_registers.read_data(),
                _ => unimplemented!(),
            }
        } else {
            self.internal_ram[address.index() % 0x0800]
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        if address >= PRG_SPACE {
            self.prg.write(address, byte);
        } else if address == OAM_DMA {
            self.write_oam_data(byte);
        } else if address == JOY1_ADDRESS {
            self.input.write(byte);
        } else if address >= APU_SPACE {
            match address {
                APU_PULSE_1_FLAGS => self.apu.write_pulse_1_flags(byte),
                APU_PULSE_1_TIMER => self.apu.write_pulse_1_timer(byte),
                APU_PULSE_1_LENGTH => self.apu.write_pulse_1_length(byte),
                APU_PULSE_2_FLAGS => self.apu.write_pulse_2_flags(byte),
                APU_PULSE_2_TIMER => self.apu.write_pulse_2_timer(byte),
                APU_PULSE_2_LENGTH => self.apu.write_pulse_2_length(byte),
                APU_TRIANGLE_FLAGS => self.apu.write_triangle_flags(byte),
                APU_TRIANGLE_TIMER => self.apu.write_triangle_timer(byte),
                APU_TRIANGLE_LENGTH => self.apu.write_triangle_length(byte),
                APU_NOISE_FLAGS => self.apu.write_noise_flags(byte),
                APU_NOISE_MODE => self.apu.write_noise_mode(byte),
                APU_NOISE_LENGTH => self.apu.write_noise_length(byte),
                APU_FRAME_COUNTER => self.apu.write_frame_counter(byte),
                APU_STATUS => self.apu.write_status(byte),
                _ => self.the_rest.write(address, byte), // TODO
            }
        } else if address >= PPU_SPACE {
            let mirrored = PPU_SPACE + (address.index() % 8) as u16;
            let ppu_registers = self.ppu_registers.borrow_mut();
            match mirrored {
                PPU_CONTROL => {
                    ppu_registers.write_control(byte);
                }
                PPU_MASK => {
                    ppu_registers.write_mask(byte);
                }
                OAM_ADDRESS => {
                    ppu_registers.write_oam_address(byte);
                }
                OAM_DATA => {
                    ppu_registers.write_oam_data(byte);
                }
                PPU_SCROLL => {
                    ppu_registers.write_scroll(byte);
                }
                PPU_ADDRESS => {
                    trace!("Writing PPU address {:#04x}", byte);
                    ppu_registers.write_address(byte);
                }
                PPU_DATA => {
                    trace!("Writing PPU data {:#04x}", byte);
                    ppu_registers.write_data(byte);
                }
                _ => {
                    // Writing to a read-only register
                    // TODO: check behaviour https://www.nesdev.org/wiki/PPU_registers
                }
            }
        } else {
            self.internal_ram[address.index() % 0x0800] = byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_and_write_internal_ram_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        for value in 0x0..=0x07ff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
        }
    }

    #[test]
    fn nes_cpu_memory_addresses_0x800_to_0x1fff_mirror_internal_ram() {
        let mut memory = nes_cpu_memory();

        for value in 0x0800..=0x1fff {
            let address = Address::new(value);
            let true_address = Address::new(value % 0x0800);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.read(true_address), value as u8);

            memory.write(true_address, (value + 1) as u8);
            assert_eq!(memory.read(address), (value + 1) as u8);
        }
    }

    #[test]
    fn can_write_ppuctrl_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2000), 0x43);
        assert_eq!(memory.ppu_registers.control, 0x43);
        memory.write(Address::new(0x3ff8), 0x44);
        assert_eq!(memory.ppu_registers.control, 0x44);
    }

    #[test]
    fn can_write_ppumask_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2001), 0x43);
        assert_eq!(memory.ppu_registers.mask, 0x43);
        memory.write(Address::new(0x3ff9), 0x44);
        assert_eq!(memory.ppu_registers.mask, 0x44);
    }

    #[test]
    fn can_read_ppustatus_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.status = 0x43;
        assert_eq!(memory.read(Address::new(0x2002)), 0x43);
        assert_eq!(memory.read(Address::new(0x3ffa)), 0x43);
    }

    #[test]
    fn can_write_oamaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2003), 0x43);
        assert_eq!(memory.ppu_registers.oam_address, 0x43);
        memory.write(Address::new(0x3ffb), 0x44);
        assert_eq!(memory.ppu_registers.oam_address, 0x44);
    }

    #[test]
    fn can_read_oamdata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.oam_data = 0x43;
        assert_eq!(memory.read(Address::new(0x3ffc)), 0x43);
        assert_eq!(memory.read(Address::new(0x2004)), 0x43);
    }

    #[test]
    fn can_write_oamdata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2004), 0x43);
        assert_eq!(memory.ppu_registers.oam_data, 0x43);
        memory.write(Address::new(0x3ffc), 0x44);
        assert_eq!(memory.ppu_registers.oam_data, 0x44);
    }

    #[test]
    fn can_write_oamdma_in_nes_cpu_memory() {
        let mut expected = [0u8; 256];
        #[allow(clippy::needless_range_loop)] // Cleaner like this
        for i in 0..=255 {
            expected[i] = i as u8;
        }

        let mut memory = nes_cpu_memory();
        for i in 0x0200..=0x02ff {
            memory.write(Address::new(i), expected[i as usize % 256]);
        }
        memory.write(Address::new(0x4014), 0x02);

        assert_eq!(memory.ppu_registers.oam_dma, expected);
    }

    #[test]
    fn can_write_ppuscroll_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2005), 0x43);
        assert_eq!(memory.ppu_registers.scroll, 0x43);
        memory.write(Address::new(0x3ffd), 0x44);
        assert_eq!(memory.ppu_registers.scroll, 0x44);
    }

    #[test]
    fn can_write_ppuaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2006), 0x43);
        assert_eq!(memory.ppu_registers.address, 0x43);
        memory.write(Address::new(0x3ffe), 0x44);
        assert_eq!(memory.ppu_registers.address, 0x44);
    }

    #[test]
    fn can_read_ppudata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x2007), 0x43);
        assert_eq!(memory.ppu_registers.data, 0x43);
        memory.write(Address::new(0x3fff), 0x44);
        assert_eq!(memory.ppu_registers.data, 0x44);
    }

    #[test]
    fn can_write_ppudata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.data = 0x43;
        assert_eq!(memory.read(Address::new(0x3fff)), 0x43);
        assert_eq!(memory.read(Address::new(0x2007)), 0x43);
    }

    #[test]
    fn can_read_and_write_cartridge_space_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        for value in 0x4020..=0xffff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.prg.read(address), value as u8);
        }
    }

    #[test]
    fn reading_from_4016_reads_from_input_device() {
        let mut memory = nes_cpu_memory();
        memory.input.0 = 24;
        assert_eq!(memory.read(Address::new(0x4016)), 24);
    }

    #[test]
    fn writing_to_4016_writes_to_input_device() {
        let mut memory = nes_cpu_memory();
        memory.write(Address::new(0x4016), 52);
        assert_eq!(memory.input.0, 52);
    }

    struct MockPPURegisters {
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

    impl PPURegisters for MockPPURegisters {
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

    fn nes_cpu_memory() -> NESCPUMemory<ArrayMemory, MockPPURegisters, MockInput> {
        let ppu = MockPPURegisters {
            control: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam_data: 0,
            scroll: 0,
            address: 0,
            data: 0,
            oam_dma: [0; 256],
        };
        let prg = ArrayMemory::default();
        let input = MockInput(0);
        NESCPUMemory::new(prg, ppu, APU::default(), input)
    }
}
