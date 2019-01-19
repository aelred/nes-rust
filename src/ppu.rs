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

impl<M> PPURegisters for PPU<M> {
    fn write_control(&mut self, byte: u8) {
        unimplemented!()
    }

    fn write_mask(&mut self, byte: u8) {
        unimplemented!()
    }

    fn read_status(&mut self) -> u8 {
        unimplemented!()
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
        unimplemented!()
    }

    fn write_address(&mut self, byte: u8) {
        unimplemented!()
    }

    fn read_data(&mut self) -> u8 {
        unimplemented!()
    }

    fn write_data(&mut self, byte: u8) {
        unimplemented!()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Color(u8);

impl Color {
    pub fn to_byte(&self) -> u8 {
        self.0
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
}
