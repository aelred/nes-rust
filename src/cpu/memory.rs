use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};

use log::trace;

use crate::Address;
use crate::ArrayMemory;
use crate::input::Input;
use crate::Memory;
use crate::ppu::PPURegisters;

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
const OAM_DMA: Address = Address::new(0x4014);
const JOY1_ADDRESS: Address = Address::new(0x4016);
const PRG_SPACE: Address = Address::new(0x4020);

pub struct NESCPUMemory<PRG, PPU, IN> {
    internal_ram: [u8; 0x800],
    prg: PRG,
    ppu_registers: PPU,
    input: IN,
    the_rest: ArrayMemory, // TODO
}

impl<PRG: Memory, PPU: PPURegisters, IN: Input> NESCPUMemory<PRG, PPU, IN> {
    pub fn new(prg: PRG, ppu_registers: PPU, input: IN) -> Self {
        NESCPUMemory {
            internal_ram: [0; 0x800],
            prg,
            ppu_registers,
            input,
            the_rest: ArrayMemory::default(),
        }
    }

    pub fn ppu_registers(&mut self) -> &mut PPU {
        &mut self.ppu_registers
    }

    pub fn input(&mut self) -> &mut IN {
        &mut self.input
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
            self.the_rest.write(address, byte) // TODO
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
                    unimplemented!();
                }
            }
        } else {
            self.internal_ram[address.index() % 0x0800] = byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::input::MockInput;
    use crate::ppu::MockPPURegisters;

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
        memory.ppu_registers.expect_write_control().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2000), 0x43);
        memory.write(Address::new(0x3ff8), 0x43);
    }

    #[test]
    fn can_write_ppumask_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_mask().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2001), 0x43);
        memory.write(Address::new(0x3ff9), 0x43);
    }

    #[test]
    fn can_read_ppustatus_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_read_status().times(2).return_const(0x43);

        assert_eq!(memory.read(Address::new(0x2002)), 0x43);
        assert_eq!(memory.read(Address::new(0x3ffa)), 0x43);
    }

    #[test]
    fn can_write_oamaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_oam_address().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2003), 0x43);
        memory.write(Address::new(0x3ffb), 0x43);
    }

    #[test]
    fn can_read_oamdata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_read_oam_data().times(2).return_const(0x43);

        assert_eq!(memory.read(Address::new(0x3ffc)), 0x43);
        assert_eq!(memory.read(Address::new(0x2004)), 0x43);
    }

    #[test]
    fn can_write_oamdata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_oam_data().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2004), 0x43);
        memory.write(Address::new(0x3ffc), 0x43);
    }

    #[test]
    fn can_write_oamdma_in_nes_cpu_memory() {
        let mut expected = [0u8; 256];
        for i in 0..=255 {
            expected[i] = i as u8;
        }

        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_oam_dma().once().with(eq(expected)).return_const(());

        for i in 0x0200..=0x02ff {
            memory.write(Address::new(i), expected[i as usize % 256]);
        }

        memory.write(Address::new(0x4014), 0x02);
    }

    #[test]
    fn can_write_ppuscroll_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_scroll().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2005), 0x43);
        memory.write(Address::new(0x3ffd), 0x43);
    }

    #[test]
    fn can_write_ppuaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_address().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2006), 0x43);
        memory.write(Address::new(0x3ffe), 0x43);
    }

    #[test]
    fn can_read_ppudata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_write_data().times(2).with(eq(0x43)).return_const(());

        memory.write(Address::new(0x2007), 0x43);
        memory.write(Address::new(0x3fff), 0x43);
    }

    #[test]
    fn can_write_ppudata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();
        memory.ppu_registers.expect_read_data().times(2).return_const(0x43);

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
        memory.input.expect_read().once().return_const(24);

        assert_eq!(memory.read(Address::new(0x4016)), 24);
    }

    #[test]
    fn writing_to_4016_writes_to_input_device() {
        let mut memory = nes_cpu_memory();
        memory.input.expect_write().once().with(eq(52)).return_const(());

        memory.write(Address::new(0x4016), 52);
    }

    fn nes_cpu_memory() -> NESCPUMemory<ArrayMemory, MockPPURegisters, MockInput> {
        let ppu = MockPPURegisters::new();
        let prg = ArrayMemory::default();
        let input = MockInput::new();
        NESCPUMemory::new(prg, ppu, input)
    }
}
