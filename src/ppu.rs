use bitflags::bitflags;

use crate::Address;
use crate::cpu::Interruptible;
use crate::Memory;

pub use self::memory::NESPPUMemory;
pub use self::registers::PPURegisters;
use self::scroll::Scroll;

mod memory;
mod registers;
mod scroll;

const BACKGROUND_PALETTES: Address = Address::new(0x3f00);

pub struct PPU<M> {
    memory: M,
    object_attribute_memory: [u8; 256],
    scanline: u16,
    cycle_count: u16,
    tile_pattern0: ShiftRegister,
    tile_pattern1: ShiftRegister,
    palette_select0: ShiftRegister,
    palette_select1: ShiftRegister,
    control: Control,
    status: Status,
    mask: Mask,
    // These are raw u16s and not Addresses because they're also used for scrolling information.
    address: u16,
    temporary_address: u16,
    write_lower: bool,
    fine_x: u8,
    oam_address: u8,
}

impl<M: Memory> PPU<M> {
    pub fn with_memory(memory: M) -> Self {
        PPU {
            memory,
            object_attribute_memory: [0; 256],
            scanline: 0,
            cycle_count: 0,
            tile_pattern0: ShiftRegister::default(),
            tile_pattern1: ShiftRegister::default(),
            palette_select0: ShiftRegister::default(),
            palette_select1: ShiftRegister::default(),
            control: Control::empty(),
            mask: Mask::empty(),
            status: Status::empty(),
            address: 0,
            temporary_address: 0,
            write_lower: false,
            fine_x: 0,
            oam_address: 0,
        }
    }

    fn address(&self) -> Address {
        Address::new(self.address)
    }

    fn address_increment(&self) -> u16 {
        if self.control.contains(Control::ADDRESS_INCREMENT) {
            32
        } else {
            1
        }
    }

    fn tile_address(&self) -> Address {
        Scroll::new(self.address).tile_address()
    }

    fn attribute_address(&self) -> Address {
        Scroll::new(self.address).attribute_address()
    }

    fn background_pattern_table_address(&self) -> Address {
        if self.control.contains(Control::BACKGROUND_PATTERN_TABLE) {
            Address::new(0x1000)
        } else {
            Address::new(0x0000)
        }
    }

    fn increment_coarse_x(&mut self) {
        let mut scroll = Scroll::new(self.address);
        scroll.increment_coarse_x();
        self.address = scroll.bits();
    }

    fn increment_fine_y(&mut self) {
        let mut scroll = Scroll::new(self.address);
        scroll.increment_fine_y();
        self.address = scroll.bits();
    }

    fn transfer_horizontal_scroll(&mut self) {
        let mut scroll = Scroll::new(self.address);
        let temporary_scroll = Scroll::new(self.temporary_address);
        scroll.set_horizontal(temporary_scroll);
        self.address = scroll.bits();
    }
}

pub struct RunningPPU<'a, M, I> {
    ppu: &'a mut PPU<M>,
    interruptible: &'a mut I,
}

impl<'a, M: Memory, I: Interruptible> RunningPPU<'a, M, I> {
    pub fn new(ppu: &'a mut PPU<M>, interruptible: &'a mut I) -> Self {
        RunningPPU { ppu, interruptible }
    }

    pub fn tick(&mut self) -> Option<Color> {
        let scroll = Scroll::new(self.ppu.address);
        let coarse_x = scroll.coarse_x();
        let fine_x = self.ppu.fine_x;
        let coarse_y = scroll.coarse_y();
        let fine_y = scroll.fine_y();

        let color = if self.ppu.scanline < 240 && self.ppu.cycle_count < 256 {
            if self.ppu.cycle_count % 8 == 0 {
                let pattern_index = self.ppu.memory.read(self.ppu.tile_address());
                let attribute_byte = self.ppu.memory.read(self.ppu.attribute_address());
                let attribute_bit_index0 = (coarse_y & 2) << 1 | (coarse_x & 2);
                let attribute_bit_index1 = attribute_bit_index0 + 1;

                let pattern_table_address = self.ppu.background_pattern_table_address();
                let index = u16::from(pattern_index) << 4 | u16::from(fine_y);
                let pattern_address0 = pattern_table_address + index;
                let pattern_address1 = pattern_address0 + 0b1000;

                self.ppu
                    .tile_pattern0
                    .set_next_byte(self.ppu.memory.read(pattern_address0));
                self.ppu
                    .tile_pattern1
                    .set_next_byte(self.ppu.memory.read(pattern_address1));

                let palette0 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index0);
                let palette1 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index1);

                self.ppu.palette_select0.set_next_byte(palette0);
                self.ppu.palette_select1.set_next_byte(palette1);

                self.ppu.increment_coarse_x();
            }

            let bit0 = u16::from(self.ppu.tile_pattern0.get_bit(fine_x));
            let bit1 = u16::from(self.ppu.tile_pattern1.get_bit(fine_x)) << 1;
            let bit2 = u16::from(self.ppu.palette_select0.get_bit(fine_x)) << 2;
            let bit3 = u16::from(self.ppu.palette_select1.get_bit(fine_x)) << 3;
            let color_index = bit0 | bit1 | bit2 | bit3;

            self.ppu.tile_pattern0.shift();
            self.ppu.tile_pattern1.shift();
            self.ppu.palette_select0.shift();
            self.ppu.palette_select1.shift();

            let address = BACKGROUND_PALETTES + color_index;

            Some(Color(self.ppu.memory.read(address)))
        } else {
            None
        };

        self.ppu.cycle_count += 1;

        if self.ppu.cycle_count == 340 {
            self.ppu.cycle_count = 0;

            if self.ppu.scanline < 240 {
                self.ppu.increment_fine_y();
                self.ppu.transfer_horizontal_scroll();
            }

            self.ppu.scanline += 1;

            if self.ppu.scanline == 241 {
                self.ppu.status.insert(Status::VBLANK);

                if self.ppu.control.contains(Control::NMI_ON_VBLANK) {
                    self.interruptible.non_maskable_interrupt();
                }
            }

            // TODO: The VBLANK is much too long
            if self.ppu.scanline == 600 {
                self.ppu.scanline = 0;
                self.ppu.status.remove(Status::VBLANK);
                self.ppu.address = self.ppu.temporary_address;
            }
        }

        color
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
struct ShiftRegister(u16);

impl ShiftRegister {
    fn set_next_byte(&mut self, byte: u8) {
        self.0 |= u16::from(byte);
    }

    fn shift(&mut self) {
        self.0 <<= 1;
    }

    fn get_bit(&self, bit: u8) -> bool {
        assert!(bit < 8);
        let mask = 0b1000_0000_0000_0000 >> bit;
        (self.0 & mask) != 0
    }
}

fn set_all_bits_to_bit_at_index(byte: u8, index: u8) -> u8 {
    (!((byte >> index) & 1)).wrapping_add(1)
}

impl<M: Memory> PPURegisters for PPU<M> {
    fn write_control(&mut self, byte: u8) {
        self.control = Control::from_bits_truncate(byte);

        // Set bits of temporary address to nametable
        self.temporary_address &= 0b1111_0011_1111_1111;
        self.temporary_address |= u16::from(self.control.nametable_select()) << 10;
    }

    fn write_mask(&mut self, byte: u8) {
        self.mask = Mask::from_bits_truncate(byte);
    }

    fn read_status(&mut self) -> u8 {
        let status = self.status;
        self.status.remove(Status::VBLANK);
        self.write_lower = false;
        status.bits()
    }

    fn write_oam_address(&mut self, byte: u8) {
        self.oam_address = byte;
    }

    fn read_oam_data(&mut self) -> u8 {
        unimplemented!()
    }

    fn write_oam_data(&mut self, _byte: u8) {
        unimplemented!()
    }

    fn write_scroll(&mut self, byte: u8) {
        let fine = byte & 0b111;
        let coarse = (byte & 0b1111_1000) >> 3;
        let mut scroll = Scroll::from_bits_truncate(self.temporary_address);

        if self.write_lower {
            scroll.set_coarse_y(coarse);
            scroll.set_fine_y(fine);
        } else {
            scroll.set_coarse_x(coarse);
            self.fine_x = fine;
        }

        self.temporary_address = scroll.bits();
        self.write_lower = !self.write_lower;
    }

    fn write_address(&mut self, byte: u8) {
        if self.write_lower {
            self.temporary_address &= 0b1111_1111_0000_0000;
            self.temporary_address |= u16::from(byte);
            self.address = self.temporary_address;
        } else {
            self.temporary_address &= 0b0000_0000_1111_1111;
            self.temporary_address |= u16::from(byte & 0b0011_1111) << 8;
        }

        self.write_lower = !self.write_lower;
    }

    fn read_data(&mut self) -> u8 {
        let byte = self.memory.read(self.address());
        self.address += self.address_increment();
        byte
    }

    fn write_data(&mut self, byte: u8) {
        self.memory.write(self.address(), byte);
        self.address += self.address_increment();
    }

    fn write_oam_dma(&mut self, bytes: [u8; 256]) {
        self.object_attribute_memory = bytes;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Color(u8);

impl Color {
    pub fn to_byte(&self) -> u8 {
        self.0
    }
}

bitflags! {
    struct Control: u8 {
        const NMI_ON_VBLANK            = 0b1000_0000;
        const PPU_MASTER_SLAVE         = 0b0100_0000;
        const SPRITE_SIZE              = 0b0010_0000;
        const BACKGROUND_PATTERN_TABLE = 0b0001_0000;
        const SPRITE_PATTERN_TABLE     = 0b0000_1000;
        const ADDRESS_INCREMENT        = 0b0000_0100;
        const NAMETABLE_SELECT         = 0b0000_0011;
    }
}

impl Control {
    fn nametable_select(self) -> u8 {
        (self & Control::NAMETABLE_SELECT).bits()
    }
}

bitflags! {
    struct Mask: u8 {
        const EMPHASIZE_BLUE       = 0b1000_0000;
        const EMPHASIZE_GREEN      = 0b0100_0000;
        const EMPHASIZE_RED        = 0b0010_0000;
        const SHOW_SPRITES         = 0b0001_0000;
        const SHOW_BACKGROUND      = 0b0000_1000;
        const SHOW_SPRITES_LEFT    = 0b0000_0100;
        const SHOW_BACKGROUND_LEFT = 0b0000_0010;
        const GREYSCALE            = 0b0000_0001;
    }
}

bitflags! {
    struct Status: u8 {
        const VBLANK          = 0b1000_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}

#[cfg(test)]
mod tests {
    use crate::Address;
    use crate::ArrayMemory;
    use crate::mem;

    use super::*;

    #[test]
    fn each_tick_produces_a_color() {
        let memory = ArrayMemory::default();
        let mut ppu = PPU::with_memory(memory);
        let mut stub = StubInterruptible;
        let mut rppu = RunningPPU::new(&mut ppu, &mut stub);
        let _color: Option<Color> = rppu.tick();
    }

    #[test]
    fn writing_ppu_control_sets_control() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b1010_1010);
        assert_eq!(ppu.control.bits(), 0b1010_1010);
    }

    #[test]
    fn writing_ppu_control_sets_temporary_address_to_nametable() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b0000_0000);
        assert_eq!(ppu.temporary_address, 0b0000_0000_0000_0000);
        ppu.write_control(0b0000_0001);
        assert_eq!(ppu.temporary_address, 0b0000_0100_0000_0000);
        ppu.write_control(0b0000_0010);
        assert_eq!(ppu.temporary_address, 0b0000_1000_0000_0000);
        ppu.write_control(0b0000_0011);
        assert_eq!(ppu.temporary_address, 0b0000_1100_0000_0000);
    }

    #[test]
    fn writing_ppu_control_sets_tile_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b0000_0000);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.tile_address(), Address::new(0x2000));
        ppu.write_control(0b0000_0001);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.tile_address(), Address::new(0x2400));
        ppu.write_control(0b0000_0010);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.tile_address(), Address::new(0x2800));
        ppu.write_control(0b0000_0011);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.tile_address(), Address::new(0x2c00));
    }

    #[test]
    fn writing_ppu_control_sets_attribute_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b0000_0000);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.attribute_address(), Address::new(0x23c0));
        ppu.write_control(0b0000_0001);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.attribute_address(), Address::new(0x27c0));
        ppu.write_control(0b0000_0010);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.attribute_address(), Address::new(0x2bc0));
        ppu.write_control(0b0000_0011);
        ppu.address = ppu.temporary_address;
        assert_eq!(ppu.attribute_address(), Address::new(0x2fc0));
    }

    #[test]
    fn writing_ppu_control_sets_background_pattern_table_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b0000_0000);
        assert_eq!(ppu.background_pattern_table_address(), Address::new(0x0000));
        ppu.write_control(0b0001_0000);
        assert_eq!(ppu.background_pattern_table_address(), Address::new(0x1000));
    }

    #[test]
    fn writing_ppu_mask_sets_mask() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_mask(0b1010_1010);
        assert_eq!(ppu.mask.bits(), 0b1010_1010);
    }

    #[test]
    fn reading_ppu_status_returns_status() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.status = Status::from_bits_truncate(0b1010_0000);
        assert_eq!(ppu.read_status(), 0b1010_0000);
    }

    #[test]
    fn reading_ppu_status_resets_vblank() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.status = Status::from_bits_truncate(0b1010_0000);

        assert_eq!(ppu.status.contains(Status::VBLANK), true);
        ppu.read_status();
        assert_eq!(ppu.status.contains(Status::VBLANK), false);
    }

    #[test]
    fn reading_ppu_status_resets_address_toggle() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_address(0x06);

        assert_eq!(ppu.temporary_address, 0x0634);

        ppu.write_address(0x00);
        ppu.write_address(0x12);
        ppu.read_status();
        ppu.write_address(0x34);
        ppu.write_address(0x56);

        assert_eq!(ppu.temporary_address, 0x3456);
    }

    #[test]
    fn writing_ppu_address_once_sets_masked_upper_bits_of_temporary_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.temporary_address = 0;
        ppu.address = 0;
        ppu.write_lower = false;

        ppu.write_address(0b1110_1010);

        assert_eq!(ppu.temporary_address, 0b0010_1010_0000_0000);
        assert_eq!(ppu.address, 0);
    }

    #[test]
    fn writing_ppu_address_twice_sets_lower_bits_of_temporary_address_and_transfers_to_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.temporary_address = 0;
        ppu.address = 0;
        ppu.write_lower = false;

        ppu.write_address(0b1110_1010);
        ppu.write_address(0b0101_0101);

        assert_eq!(ppu.temporary_address, 0b0010_1010_0101_0101);
        assert_eq!(ppu.address, 0b0010_1010_0101_0101);
    }

    #[test]
    fn writing_ppu_address_twice_then_reading_data_reads_data_from_address() {
        let mut ppu = PPU::with_memory(mem!(0x1234 => 0x54));

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        assert_eq!(ppu.read_data(), 0x54);
    }

    #[test]
    fn writing_ppu_address_twice_then_writing_data_writes_data_to_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_data(0x54);
        assert_eq!(ppu.memory.read(Address::new(0x1234)), 0x54);
    }

    #[test]
    fn reading_or_writing_ppu_data_increments_address_by_increment_in_control_register() {
        let mut ppu = PPU::with_memory(mem! {
            0x1234 => { 0x00, 0x64, 0x00 }
            0x1254 => { 0x84 }
            0x1274 => { 0x00 }
        });

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_control(0b0000_0000);

        ppu.write_data(0x54);
        assert_eq!(ppu.read_data(), 0x64);
        ppu.write_data(0x74);
        assert_eq!(ppu.memory.read(Address::new(0x1236)), 0x74);

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_control(0b0000_0100);

        ppu.write_data(0x74);
        assert_eq!(ppu.read_data(), 0x84);
        ppu.write_data(0x94);
        assert_eq!(ppu.memory.read(Address::new(0x1274)), 0x94);
    }

    #[test]
    fn writing_oam_address_sets_oam_address() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_oam_address(0x42);

        assert_eq!(ppu.oam_address, 0x42);
    }

    #[test]
    fn writing_oam_dma_writes_from_cpu_page_to_oam() {
        let mut ppu = PPU::with_memory(mem!());

        let mut data = [0; 256];

        for (i, elem) in data.iter_mut().enumerate() {
            *elem = i as u8;
        }

        ppu.write_oam_dma(data);

        assert_eq!(ppu.object_attribute_memory.to_vec(), data.to_vec());
    }

    #[test]
    fn writing_ppu_scroll_writes_to_temporary_register() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_scroll(0b1111_1101);
        assert_eq!(ppu.temporary_address, 0b1_1111);
        assert_eq!(ppu.fine_x, 0b101);

        ppu.write_scroll(0b1010_1111);
        assert_eq!(ppu.temporary_address, 0b0111_0010_1011_1111);
        assert_eq!(ppu.fine_x, 0b101);
    }

    #[test]
    fn incrementing_coarse_x_increments_to_next_tile() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0x41;
        ppu.increment_coarse_x();
        assert_eq!(ppu.address, 0x42);
    }

    #[test]
    fn incrementing_coarse_x_switches_to_next_horizontal_nametable_when_coarse_x_is_31() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0b1000_0001_1111;
        ppu.increment_coarse_x();
        assert_eq!(ppu.address, 0b1100_0000_0000);

        ppu.address = 0b1100_0001_1111;
        ppu.increment_coarse_x();
        assert_eq!(ppu.address, 0b1000_0000_0000);
    }

    #[test]
    fn incrementing_fine_y_increments_fine_y_by_1() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0b0011_0000_0000_0000;
        ppu.increment_fine_y();
        assert_eq!(ppu.address, 0b0100_0000_0000_0000);
    }

    #[test]
    fn incrementing_fine_y_increments_coarse_y_when_fine_y_is_7() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0b0111_0000_0010_0000;
        ppu.increment_fine_y();
        assert_eq!(ppu.address, 0b0000_0000_0100_0000);
    }

    #[test]
    fn incrementing_fine_y_switches_to_next_vertical_nametable_when_coarse_y_is_29() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0b0111_0011_1010_0000;
        ppu.increment_fine_y();
        assert_eq!(ppu.address, 0b0000_1000_0000_0000);

        ppu.address = 0b0111_1011_1010_0000;
        ppu.increment_fine_y();
        assert_eq!(ppu.address, 0b0000_0000_0000_0000);
    }

    #[test]
    fn transfer_horizontal_scroll_transfers_horizontal_scroll_from_temporary_to_address() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.address = 0b0010_1010_1010_1010;
        ppu.temporary_address = 0b0100_0100_0101_0101;

        ppu.transfer_horizontal_scroll();

        assert_eq!(ppu.address, 0b0010_1110_1011_0101);
    }

    #[test]
    fn can_get_tile_address_from_scroll() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0x0ABC;

        assert_eq!(ppu.tile_address(), Address::new(0x2ABC));
    }

    #[test]
    fn can_get_attribute_address_from_scroll() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.address = 0b0000_0000_0000_0000;
        assert_eq!(ppu.attribute_address(), Address::new(0b0010_0011_1100_0000));

        ppu.address = 0b0000_0000_0001_1100;
        assert_eq!(ppu.attribute_address(), Address::new(0b0010_0011_1100_0111));

        ppu.address = 0b0000_0011_1000_0000;
        assert_eq!(ppu.attribute_address(), Address::new(0b0010_0011_1111_1000));

        ppu.address = 0b0000_1100_0000_0000;
        assert_eq!(ppu.attribute_address(), Address::new(0b0010_1111_1100_0000));
    }

    struct StubInterruptible;

    impl Interruptible for StubInterruptible {
        fn non_maskable_interrupt(&mut self) {}
    }
}
