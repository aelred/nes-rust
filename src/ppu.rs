use bitflags::bitflags;

use crate::Address;
use crate::cpu::Interruptible;
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

    fn address_increment(&self) -> u16 {
        if self.control.contains(Control::ADDRESS_INCREMENT) {
            32
        } else {
            1
        }
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

    pub fn tick(&mut self) -> Color {
        if self.ppu.cycle_count % 8 == 0 {
            let coarse_x = self.ppu.horizontal_scroll;
            let coarse_y = (self.ppu.vertical_scroll / 8) as u8;

            let tile_index = coarse_x as u16 + coarse_y as u16 * 32;
            let attribute_index = ((coarse_y / 4) & 0b111) << 3 | (coarse_x / 4) & 0b111;

            let pattern_index = self.ppu.memory.read(NAMETABLES + u16::from(tile_index));
            let attribute_byte = self
                .ppu
                .memory
                .read(ATTRIBUTE_TABLE + u16::from(attribute_index));
            let attribute_bit_index0 =
                (((tile_index >> 1) & (0b1 + (tile_index >> 5)) & 0b10) * 2) as u8;
            let attribute_bit_index1 = attribute_bit_index0 + 1;

            let pattern_address0 = Address::new(
                0x1000 | u16::from(pattern_index) << 4 | (self.ppu.vertical_scroll & 0b0111),
            );
            let pattern_address1 = pattern_address0 + 0b1000;

            self.ppu.tile_pattern0 |= u16::from(self.ppu.memory.read(pattern_address0));
            self.ppu.tile_pattern1 |= u16::from(self.ppu.memory.read(pattern_address1));

            let palette0 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index0);
            let palette1 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index1);

            self.ppu.palette_select0 |= u16::from(palette0);
            self.ppu.palette_select1 |= u16::from(palette1);

            self.ppu.horizontal_scroll += 1;

            if self.ppu.horizontal_scroll == 32 {
                self.ppu.horizontal_scroll = 0;
                self.ppu.vertical_scroll += 1;

                if self.ppu.vertical_scroll == 220 {
                    self.ppu.status.insert(Status::VBLANK);

                    if self.ppu.control.contains(Control::NMI_ON_VBLANK) {
                        self.interruptible.non_maskable_interrupt();
                    }
                }

                if self.ppu.vertical_scroll == 240 {
                    self.ppu.vertical_scroll = 0;
                    self.ppu.status.remove(Status::VBLANK);
                }
            }
        }

        self.ppu.cycle_count = self.ppu.cycle_count.wrapping_add(1);

        let mask = 0b1000_0000_0000_0000;
        let bit0 = (self.ppu.tile_pattern0 & mask) >> 15;
        let bit1 = (self.ppu.tile_pattern1 & mask) >> 14;
        let bit2 = (self.ppu.palette_select0 & mask) >> 13;
        let bit3 = (self.ppu.palette_select1 & mask) >> 12;
        let color_index = bit0 | bit1 | bit2 | bit3;

        self.ppu.tile_pattern0 <<= 1;
        self.ppu.tile_pattern1 <<= 1;
        self.ppu.palette_select0 <<= 1;
        self.ppu.palette_select1 <<= 1;

        let address = BACKGROUND_PALETTES + color_index;

        Color(self.ppu.memory.read(address))
    }
}

fn set_all_bits_to_bit_at_index(byte: u8, index: u8) -> u8 {
    (!((byte >> index) & 1)).wrapping_add(1)
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

impl<'a, T: PPURegisters> PPURegisters for &'a mut T {
    fn write_control(&mut self, byte: u8) {
        (*self).write_control(byte)
    }

    fn write_mask(&mut self, byte: u8) {
        (*self).write_mask(byte)
    }

    fn read_status(&mut self) -> u8 {
        (*self).read_status()
    }

    fn write_oam_address(&mut self, byte: u8) {
        (*self).write_oam_address(byte)
    }

    fn read_oam_data(&mut self) -> u8 {
        (*self).read_oam_data()
    }

    fn write_oam_data(&mut self, byte: u8) {
        (*self).write_oam_data(byte)
    }

    fn write_scroll(&mut self, byte: u8) {
        (*self).write_scroll(byte)
    }

    fn write_address(&mut self, byte: u8) {
        (*self).write_address(byte)
    }

    fn read_data(&mut self) -> u8 {
        (*self).read_data()
    }

    fn write_data(&mut self, byte: u8) {
        (*self).write_data(byte)
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
        // TODO
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
        let mut stub = StubInterruptible;
        let mut rppu = RunningPPU::new(&mut ppu, &mut stub);
        let _color: Color = rppu.tick();
    }

    #[ignore]
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
        let mut stub = StubInterruptible;
        let mut rppu = RunningPPU::new(&mut ppu, &mut stub);

        rppu.ppu.tile_pattern0 = 0xf1;
        rppu.ppu.tile_pattern1 = 0xf0;
        rppu.ppu.palette_select0 = 0xf0;
        rppu.ppu.palette_select1 = 0xf1;
        assert_eq!(rppu.tick(), Color(0xaa));

        rppu.ppu.tile_pattern0 = 0xf0;
        rppu.ppu.tile_pattern1 = 0xf1;
        rppu.ppu.palette_select0 = 0xf1;
        rppu.ppu.palette_select1 = 0xf0;
        assert_eq!(rppu.tick(), Color(0xa7));

        rppu.ppu.tile_pattern0 = 0xf1;
        rppu.ppu.tile_pattern1 = 0xf0;
        rppu.ppu.palette_select0 = 0xf1;
        rppu.ppu.palette_select1 = 0xf0;
        assert_eq!(rppu.tick(), Color(0xa6));
    }

    #[ignore]
    #[test]
    fn each_tick_tile_pattern_and_palette_select_registers_shift_right() {
        let memory = ArrayMemory::default();
        let mut ppu = PPU::with_memory(memory);
        let mut stub = StubInterruptible;
        let mut rppu = RunningPPU::new(&mut ppu, &mut stub);

        rppu.ppu.tile_pattern0 = 0b1000_0000_0000_0001;
        rppu.ppu.tile_pattern1 = 0b0101_0101_0101_0101;
        rppu.ppu.palette_select0 = 0b1111_1111_1111_1111;
        rppu.ppu.palette_select1 = 0b0000_0000_1111_1111;

        rppu.tick();

        assert_eq!(rppu.ppu.tile_pattern0, 0b0100_0000_0000_0000);
        assert_eq!(rppu.ppu.tile_pattern1, 0b0010_1010_1010_1010);
        assert_eq!(rppu.ppu.palette_select0, 0b0111_1111_1111_1111);
        assert_eq!(rppu.ppu.palette_select1, 0b0000_0000_0111_1111);
    }

    #[ignore]
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
        let mut stub = StubInterruptible;
        let mut rppu = RunningPPU::new(&mut ppu, &mut stub);

        // Point PPU at 11th pixel row, 6nd column of nametable 0
        rppu.ppu.horizontal_scroll = 5;
        rppu.ppu.vertical_scroll = 10;

        rppu.ppu.tile_pattern0 = 0b1000_0000_0000_0001;
        rppu.ppu.tile_pattern1 = 0b0101_0101_0010_0010;
        rppu.ppu.palette_select0 = 0b1111_1111_0000_0000;
        rppu.ppu.palette_select1 = 0b0000_0000_1111_1111;

        for _ in 0..8 {
            rppu.tick();
        }

        assert_eq!(
            rppu.ppu.tile_pattern0, 0b1001_1001_1000_0000,
            "{:#b}",
            rppu.ppu.tile_pattern0
        );
        assert_eq!(
            rppu.ppu.tile_pattern1, 0b0110_0110_0101_0101,
            "{:#b}",
            rppu.ppu.tile_pattern1
        );
        assert_eq!(
            rppu.ppu.palette_select0, 0b0000_0000_1111_1111,
            "{:#b}",
            rppu.ppu.palette_select0
        );
        assert_eq!(
            rppu.ppu.palette_select1, 0b1111_1111_0000_0000,
            "{:#b}",
            rppu.ppu.palette_select1
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

    struct StubInterruptible;

    impl Interruptible for StubInterruptible {
        fn non_maskable_interrupt(&mut self) {}
    }
}
