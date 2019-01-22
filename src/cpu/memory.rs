use std::borrow::BorrowMut;
use std::marker::PhantomData;

use crate::Address;
use crate::ArrayMemory;
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
const PRG_SPACE: Address = Address::new(0x4020);

pub struct NESCPUMemory<PRG> {
    internal_ram: [u8; 0x800],
    prg: PRG,
    the_rest: ArrayMemory, // TODO
}

impl<PRG> NESCPUMemory<PRG> {
    pub fn new(prg: PRG) -> Self {
        NESCPUMemory {
            internal_ram: [0; 0x800],
            prg,
            the_rest: ArrayMemory::default(),
        }
    }
}

pub struct RunningNESCPUMemory<M, PRG, PPU> {
    memory: M,
    ppu_registers: PPU,
    phantom: PhantomData<PRG>,
}

impl<M, PRG, PPU> RunningNESCPUMemory<M, PRG, PPU> {
    pub fn new(memory: M, ppu_registers: PPU) -> Self {
        RunningNESCPUMemory {
            memory,
            ppu_registers,
            phantom: PhantomData,
        }
    }
}

impl<M: BorrowMut<NESCPUMemory<PRG>>, PRG: Memory, PPU: PPURegisters>
RunningNESCPUMemory<M, PRG, PPU>
{
    fn write_oam_data(&mut self, page: u8) {
        let address = Address::from_bytes(page, 0);

        let mut data = [0; 256];

        for (offset, byte) in data.iter_mut().enumerate() {
            *byte = self.read(address + offset as u16);
        }

        self.ppu_registers.write_oam_dma(data);
    }
}

impl<M: BorrowMut<NESCPUMemory<PRG>>, PRG: Memory, PPU: PPURegisters> Memory
for RunningNESCPUMemory<M, PRG, PPU>
{
    fn read(&mut self, address: Address) -> u8 {
        if address >= PRG_SPACE {
            self.memory.borrow_mut().prg.read(address)
        } else if address >= APU_SPACE {
            self.memory.borrow_mut().the_rest.read(address) // TODO
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
            self.memory.borrow_mut().internal_ram[address.index() % 0x0800]
        }
    }

    fn write(&mut self, address: Address, byte: u8) {
        if address >= PRG_SPACE {
            self.memory.borrow_mut().prg.write(address, byte);
        } else if address == OAM_DMA {
            self.write_oam_data(byte);
        } else if address >= APU_SPACE {
            self.memory.borrow_mut().the_rest.write(address, byte) // TODO
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
                    ppu_registers.write_address(byte);
                }
                PPU_DATA => {
                    ppu_registers.write_data(byte);
                }
                _ => {
                    unimplemented!();
                }
            }
        } else {
            self.memory.borrow_mut().internal_ram[address.index() % 0x0800] = byte;
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
        assert_eq!(memory.ppu_registers.control, Some(0x43));

        memory.write(Address::new(0x3ff8), 0x44);
        assert_eq!(memory.ppu_registers.control, Some(0x44));
    }

    #[test]
    fn can_write_ppumask_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2001), 0x43);
        assert_eq!(memory.ppu_registers.mask, Some(0x43));

        memory.write(Address::new(0x3ff9), 0x44);
        assert_eq!(memory.ppu_registers.mask, Some(0x44));
    }

    #[test]
    fn can_read_ppustatus_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.ppu_registers.status = Some(0x43);
        assert_eq!(memory.read(Address::new(0x2002)), 0x43);
        assert_eq!(memory.read(Address::new(0x3ffa)), 0x43);
    }

    #[test]
    fn can_write_oamaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2003), 0x43);
        assert_eq!(memory.ppu_registers.oam_address, Some(0x43));

        memory.write(Address::new(0x3ffb), 0x44);
        assert_eq!(memory.ppu_registers.oam_address, Some(0x44));
    }

    #[test]
    fn can_read_and_write_oamdata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2004), 0x43);
        assert_eq!(memory.read(Address::new(0x3ffc)), 0x43);
        assert_eq!(memory.ppu_registers.oam_data, Some(0x43));

        memory.write(Address::new(0x3ffc), 0x44);
        assert_eq!(memory.read(Address::new(0x2004)), 0x44);
        assert_eq!(memory.ppu_registers.oam_data, Some(0x44));
    }

    #[test]
    fn can_write_oamdma_in_nes_cpu_memory() {
        let mut expected = Vec::new();
        for i in 0..=255 {
            expected.push(i);
        }

        let mut memory = nes_cpu_memory();

        for i in 0x0200..=0x02ff {
            memory.write(Address::new(i), expected[i as usize % 256]);
        }

        memory.write(Address::new(0x4014), 0x02);

        assert_eq!(
            memory.ppu_registers.oam_dma,
            Some(expected.into_boxed_slice())
        );
    }

    #[test]
    fn can_write_ppuscroll_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2005), 0x43);
        assert_eq!(memory.ppu_registers.scroll, Some(0x43));

        memory.write(Address::new(0x3ffd), 0x44);
        assert_eq!(memory.ppu_registers.scroll, Some(0x44));
    }

    #[test]
    fn can_write_ppuaddr_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2006), 0x43);
        assert_eq!(memory.ppu_registers.address, Some(0x43));

        memory.write(Address::new(0x3ffe), 0x44);
        assert_eq!(memory.ppu_registers.address, Some(0x44));
    }

    #[test]
    fn can_read_and_write_ppudata_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        memory.write(Address::new(0x2007), 0x43);
        assert_eq!(memory.read(Address::new(0x3fff)), 0x43);
        assert_eq!(memory.ppu_registers.data, Some(0x43));

        memory.write(Address::new(0x3fff), 0x44);
        assert_eq!(memory.read(Address::new(0x2007)), 0x44);
        assert_eq!(memory.ppu_registers.data, Some(0x44));
    }

    #[test]
    fn can_read_and_write_cartridge_space_in_nes_cpu_memory() {
        let mut memory = nes_cpu_memory();

        for value in 0x4020..=0xffff {
            let address = Address::new(value);

            memory.write(address, value as u8);
            assert_eq!(memory.read(address), value as u8);
            assert_eq!(memory.memory.prg.read(address), value as u8);
        }
    }

    type TestNESCPUMemory = super::NESCPUMemory<ArrayMemory>;

    fn nes_cpu_memory() -> RunningNESCPUMemory<TestNESCPUMemory, ArrayMemory, MockPPURegisters> {
        let ppu = MockPPURegisters::default();
        let prg = ArrayMemory::default();
        let memory = NESCPUMemory::new(prg);
        RunningNESCPUMemory::new(memory, ppu)
    }

    #[derive(Default)]
    struct MockPPURegisters {
        control: Option<u8>,
        mask: Option<u8>,
        status: Option<u8>,
        oam_address: Option<u8>,
        oam_data: Option<u8>,
        scroll: Option<u8>,
        address: Option<u8>,
        data: Option<u8>,
        oam_dma: Option<Box<[u8]>>,
    }

    impl PPURegisters for MockPPURegisters {
        fn write_control(&mut self, byte: u8) {
            self.control = Some(byte);
        }

        fn write_mask(&mut self, byte: u8) {
            self.mask = Some(byte);
        }

        fn read_status(&mut self) -> u8 {
            self.status.unwrap()
        }

        fn write_oam_address(&mut self, byte: u8) {
            self.oam_address = Some(byte);
        }

        fn read_oam_data(&mut self) -> u8 {
            self.oam_data.unwrap()
        }

        fn write_oam_data(&mut self, byte: u8) {
            self.oam_data = Some(byte);
        }

        fn write_scroll(&mut self, byte: u8) {
            self.scroll = Some(byte);
        }

        fn write_address(&mut self, byte: u8) {
            self.address = Some(byte);
        }

        fn read_data(&mut self) -> u8 {
            self.data.unwrap()
        }

        fn write_data(&mut self, byte: u8) {
            self.data = Some(byte);
        }

        fn write_oam_dma(&mut self, bytes: [u8; 256]) {
            self.oam_dma = Some(Box::new(bytes));
        }
    }
}
