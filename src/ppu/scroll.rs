use bitflags::bitflags;

use crate::Address;

bitflags! {
    pub struct Scroll: u16 {
        const COARSE_X             = 0b0000_0000_0001_1111;
        const COARSE_Y             = 0b0000_0011_1110_0000;
        const HORIZONTAL_NAMETABLE = 0b0000_0100_0000_0000;
        const VERTICAL_NAMETABLE   = 0b0000_1000_0000_0000;
        const FINE_Y               = 0b0111_0000_0000_0000;
        const HORIZONTAL           = Self::COARSE_X.bits | Self::HORIZONTAL_NAMETABLE.bits;
        const VERTICAL             = Self::COARSE_Y.bits | Self::VERTICAL_NAMETABLE.bits;
        const COARSE               = Self::COARSE_X.bits | Self::COARSE_Y.bits;
        const NAMETABLE_SELECT     = Self::HORIZONTAL_NAMETABLE.bits | Self::VERTICAL_NAMETABLE.bits;
        const TILE_INDEX           = Self::HORIZONTAL.bits | Self::VERTICAL.bits;
    }
}

impl Scroll {
    pub fn new(bits: u16) -> Scroll {
        Self::from_bits_truncate(bits)
    }

    pub fn coarse_x(self) -> u8 {
        (self & Self::COARSE_X).bits() as u8
    }

    pub fn set_coarse_x(&mut self, coarse_x: u8) {
        self.remove(Scroll::COARSE_X);
        self.bits |= 0b1_1111 & u16::from(coarse_x);
    }

    pub fn coarse_y(self) -> u8 {
        ((self & Self::COARSE_Y).bits() >> 5) as u8
    }

    pub fn set_coarse_y(&mut self, coarse_y: u8) {
        self.remove(Scroll::COARSE_Y);
        self.bits |= (0b1_1111 & u16::from(coarse_y)) << 5;
    }

    pub fn fine_y(self) -> u8 {
        ((self & Self::FINE_Y).bits() >> 12) as u8
    }

    pub fn set_fine_y(&mut self, fine_y: u8) {
        self.remove(Scroll::FINE_Y);
        self.bits |= (0b111 & u16::from(fine_y)) << 12;
    }

    pub fn tile_address(self) -> Address {
        let index = (self & Scroll::TILE_INDEX).bits();
        Address::new(0x2000 | index)
    }

    pub fn attribute_address(self) -> Address {
        let nametable = self & Scroll::NAMETABLE_SELECT;
        let upper_coarse_x = u16::from(self.coarse_x() & 0b1_1100) >> 2;
        let upper_coarse_y = u16::from(self.coarse_y() & 0b1_1100) << 1;
        Address::new(0x23C0 | nametable.bits() | upper_coarse_x | upper_coarse_y)
    }

    pub fn increment_coarse_x(&mut self) {
        if self.coarse_x() != 31 {
            self.bits += 1;
        } else {
            self.remove(Scroll::COARSE_X);
            self.toggle(Scroll::HORIZONTAL_NAMETABLE);
        }
    }

    pub fn increment_fine_y(&mut self) {
        if self.fine_y() != 7 {
            self.bits += 0b0001_0000_0000_0000;
        } else {
            self.remove(Scroll::FINE_Y);

            if self.coarse_y() != 29 {
                self.bits += 0b0000_0000_0010_0000;
            } else {
                self.remove(Scroll::COARSE_Y);
                self.toggle(Scroll::VERTICAL_NAMETABLE);
            }
        }
    }

    pub fn set_horizontal(&mut self, from: Scroll) {
        self.remove(Scroll::HORIZONTAL);
        self.insert(from & Scroll::HORIZONTAL);
    }
}
