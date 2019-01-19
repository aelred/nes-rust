use std::cell::RefCell;
use std::rc::Rc;

use bitflags::bitflags;

use crate::Address;
use crate::Memory;

const NAMETABLES: Address = Address::new(0x2000);
const ATTRIBUTE_TABLE: Address = Address::new(0x23c0);
const BACKGROUND_PALETTES: Address = Address::new(0x3f00);

pub struct PPU<M> {
    memory: M,
    horizontal_scroll: u8,
    vertical_scroll: u16,
    cycle_count: u8,
    tile_pattern0: u16,
    tile_pattern1: u16,
    palette_select0: u16,
    palette_select1: u16,
    control: Control,
    status: Status,
    mask: Mask,
    address: Address,
    write_lower: bool,
}

impl<M: Memory> PPU<M> {
    pub fn with_memory(memory: M) -> Self {
        PPU {
            memory,
            horizontal_scroll: 0,
            vertical_scroll: 0,
            cycle_count: 0,
            tile_pattern0: 0,
            tile_pattern1: 0,
            palette_select0: 0,
            palette_select1: 0,
            control: Control::empty(),
            mask: Mask::empty(),
            status: Status::empty(),
            address: Address::new(0),
            write_lower: false,
        }
    }

    pub fn tick(&mut self) -> Color {
        let bit0 = self.tile_pattern0 & 1;
        let bit1 = (self.tile_pattern1 & 1) << 1;
        let bit2 = (self.palette_select0 & 1) << 2;
        let bit3 = (self.palette_select1 & 1) << 3;
        let color_index = bit0 | bit1 | bit2 | bit3;

        self.tile_pattern0 >>= 1;
        self.tile_pattern1 >>= 1;
        self.palette_select0 >>= 1;
        self.palette_select1 >>= 1;

        let address = BACKGROUND_PALETTES + color_index;

        self.cycle_count = self.cycle_count.wrapping_add(1);

        if self.cycle_count % 8 == 0 {
            let coarse_x = self.horizontal_scroll;
            let coarse_y = (self.vertical_scroll / 8) as u8;

            let tile_index = coarse_x + coarse_y as u8 * 32;
            let attribute_index = ((coarse_y / 4) & 0b111) << 3 | (coarse_x / 4) & 0b111;

            let pattern_index = self.memory.read(NAMETABLES + u16::from(tile_index));
            let attribute_byte = self
                .memory
                .read(ATTRIBUTE_TABLE + u16::from(attribute_index));
            let attribute_bit_index0 = ((tile_index >> 1) & (0b1 + (tile_index >> 5)) & 0b10) * 2;
            let attribute_bit_index1 = attribute_bit_index0 + 1;

            let pattern_address0 =
                Address::new(u16::from(pattern_index) << 4) + self.vertical_scroll % 8;
            let pattern_address1 = pattern_address0 + 8;

            self.tile_pattern0 |= u16::from(self.memory.read(pattern_address0)) << 8;
            self.tile_pattern1 |= u16::from(self.memory.read(pattern_address1)) << 8;

            let palette0 = Self::set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index0);
            let palette1 = Self::set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index1);

            self.palette_select0 |= u16::from(palette0) << 8;
            self.palette_select1 |= u16::from(palette1) << 8;

            self.horizontal_scroll = self.horizontal_scroll.wrapping_add(1);
        }

        Color(self.memory.read(address))
    }

    fn set_all_bits_to_bit_at_index(byte: u8, index: u8) -> u8 {
        (!((byte >> index) & 1)).wrapping_add(1)
    }

    fn address_increment(&self) -> u16 {
        if self.control.contains(Control::ADDRESS_INCREMENT) {
            32
        } else {
            1
        }
    }
}

pub trait PPURegisters {
    fn write_control(&mut self, byte: u8);

    fn write_mask(&mut self, byte: u8);

    fn read_status(&mut self) -> u8;

    fn write_oam_address(&mut self, byte: u8);

    fn read_oam_data(&mut self) -> u8;

    fn write_oam_data(&mut self, byte: u8);

    fn write_scroll(&mut self, byte: u8);

    fn write_address(&mut self, byte: u8);

    fn read_data(&mut self) -> u8;

    fn write_data(&mut self, byte: u8);
}

impl<T: PPURegisters> PPURegisters for Rc<RefCell<T>> {
    fn write_control(&mut self, byte: u8) {
        self.borrow_mut().write_control(byte)
    }

    fn write_mask(&mut self, byte: u8) {
        self.borrow_mut().write_mask(byte)
    }

    fn read_status(&mut self) -> u8 {
        self.borrow_mut().read_status()
    }

    fn write_oam_address(&mut self, byte: u8) {
        self.borrow_mut().write_oam_address(byte)
    }

    fn read_oam_data(&mut self) -> u8 {
        self.borrow_mut().read_oam_data()
    }

    fn write_oam_data(&mut self, byte: u8) {
        self.borrow_mut().write_oam_data(byte)
    }

    fn write_scroll(&mut self, byte: u8) {
        self.borrow_mut().write_scroll(byte)
    }

    fn write_address(&mut self, byte: u8) {
        self.borrow_mut().write_address(byte)
    }

    fn read_data(&mut self) -> u8 {
        self.borrow_mut().read_data()
    }

    fn write_data(&mut self, byte: u8) {
        self.borrow_mut().write_data(byte)
    }
}

impl<M: Memory> PPURegisters for PPU<M> {
    fn write_control(&mut self, byte: u8) {
        self.control = Control::from_bits_truncate(byte);
    }

    fn write_mask(&mut self, byte: u8) {
        self.mask = Mask::from_bits_truncate(byte);
    }

    fn read_status(&mut self) -> u8 {
        let status = self.status.clone();
        self.status.remove(Status::VBLANK);
        self.write_lower = false;
        status.bits()
    }

    fn write_oam_address(&mut self, byte: u8) {
        unimplemented!()
    }

    fn read_oam_data(&mut self) -> u8 {
        unimplemented!()
    }

    fn write_oam_data(&mut self, byte: u8) {
        unimplemented!()
    }

    fn write_scroll(&mut self, byte: u8) {
        // TODO
    }

    fn write_address(&mut self, byte: u8) {
        self.address = if self.write_lower {
            Address::from_bytes(self.address.higher(), byte)
        } else {
            Address::from_bytes(byte, self.address.lower())
        };

        self.write_lower = !self.write_lower;
    }

    fn read_data(&mut self) -> u8 {
        let byte = self.memory.read(self.address);
        self.address += self.address_increment();
        byte
    }

    fn write_data(&mut self, byte: u8) {
        self.memory.write(self.address, byte);
        self.address += self.address_increment();
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
        let _color: Color = ppu.tick();
    }

    #[test]
    fn color_is_read_using_background_tile_bitmap_and_palette() {
        let memory = mem! {
            0x3f00 => {
                0xa1, 0xa2, 0xa3, 0xa4,
                0xa5, 0xa6, 0xa7, 0xa8,
                0xa9, 0xaa, 0xab, 0xac,
                0xad, 0xae, 0xaf, 0xb0
            }
        };

        let mut ppu = PPU::with_memory(memory);

        ppu.tile_pattern0 = 0xf1;
        ppu.tile_pattern1 = 0xf0;
        ppu.palette_select0 = 0xf0;
        ppu.palette_select1 = 0xf1;
        assert_eq!(ppu.tick(), Color(0xaa));

        ppu.tile_pattern0 = 0xf0;
        ppu.tile_pattern1 = 0xf1;
        ppu.palette_select0 = 0xf1;
        ppu.palette_select1 = 0xf0;
        assert_eq!(ppu.tick(), Color(0xa7));

        ppu.tile_pattern0 = 0xf1;
        ppu.tile_pattern1 = 0xf0;
        ppu.palette_select0 = 0xf1;
        ppu.palette_select1 = 0xf0;
        assert_eq!(ppu.tick(), Color(0xa6));
    }

    #[test]
    fn each_tick_tile_pattern_and_palette_select_registers_shift_right() {
        let memory = ArrayMemory::default();
        let mut ppu = PPU::with_memory(memory);

        ppu.tile_pattern0 = 0b1000_0000_0000_0001;
        ppu.tile_pattern1 = 0b0101_0101_0101_0101;
        ppu.palette_select0 = 0b1111_1111_1111_1111;
        ppu.palette_select1 = 0b0000_0000_1111_1111;

        ppu.tick();

        assert_eq!(ppu.tile_pattern0, 0b0100_0000_0000_0000);
        assert_eq!(ppu.tile_pattern1, 0b0010_1010_1010_1010);
        assert_eq!(ppu.palette_select0, 0b0111_1111_1111_1111);
        assert_eq!(ppu.palette_select1, 0b0000_0000_0111_1111);
    }

    #[test]
    fn every_eight_ticks_tile_pattern_and_palette_select_registers_are_read_from_memory() {
        let memory = mem! {
            // Third row of 4th tile pattern, bit 0
            0x0042 => {
                0b1001_1001
            }
            // Third row of 4th tile pattern, bit 1
            0x004a => {
                0b0110_0110
            }
            // 2nd row of nametable 0, tile pattern for 6th column
            0x2020 => {
                0x00, 0x00, 0x00, 0x00, 0x00, 0x04
            }
            // 2nd row of nametable 0, palette for 6th column
            0x23c0 => {
                0b0000_0000, 0b0010_0000
            }
        };

        let mut ppu = PPU::with_memory(memory);

        // Point PPU at 11th pixel row, 6nd column of nametable 0
        ppu.horizontal_scroll = 5;
        ppu.vertical_scroll = 10;

        ppu.tile_pattern0 = 0b1000_0000_0000_0001;
        ppu.tile_pattern1 = 0b0101_0101_0010_0010;
        ppu.palette_select0 = 0b1111_1111_0000_0000;
        ppu.palette_select1 = 0b0000_0000_1111_1111;

        for _ in 0..8 {
            ppu.tick();
        }

        assert_eq!(
            ppu.tile_pattern0, 0b1001_1001_1000_0000,
            "{:#b}",
            ppu.tile_pattern0
        );
        assert_eq!(
            ppu.tile_pattern1, 0b0110_0110_0101_0101,
            "{:#b}",
            ppu.tile_pattern1
        );
        assert_eq!(
            ppu.palette_select0, 0b0000_0000_1111_1111,
            "{:#b}",
            ppu.palette_select0
        );
        assert_eq!(
            ppu.palette_select1, 0b1111_1111_0000_0000,
            "{:#b}",
            ppu.palette_select1
        );
    }

    #[test]
    fn writing_ppu_control_sets_control() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b1010_1010);
        assert_eq!(ppu.control.bits(), 0b1010_1010);
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
        ppu.write_address(0x56);

        assert_eq!(ppu.address, Address::new(0x5634));

        ppu.write_address(0x00);
        ppu.write_address(0x12);
        ppu.read_status();
        ppu.write_address(0x34);
        ppu.write_address(0x56);

        assert_eq!(ppu.address, Address::new(0x3456));
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
}
