use std::fmt::{Debug, Formatter};

use bitflags::bitflags;
use control::PatternTable;
use control::SpriteSize;
use log::warn;
pub use registers::PPURegisters;

use crate::Address;
use crate::Memory;

use self::control::Control;
use self::mask::Mask;
pub use self::memory::NESPPUMemory;
use self::scroll::Scroll;
use self::status::Status;

mod control;
mod mask;
mod memory;
mod registers;
mod scroll;
mod status;

const BACKGROUND_PALETTES: Address = Address::new(0x3f00);
const SPRITE_PALETTES: Address = Address::new(0x3f10);

const ACTIVE_SPRITES: usize = 8;

pub struct PPU<M = NESPPUMemory> {
    memory: M,
    read_buffer: u8,
    object_attribute_memory: [u8; 256],
    scanline: u16,
    cycle_count: u16,
    tile_pattern: ShiftRegister,
    palette_select: ShiftRegister,
    active_sprites: [ActiveSprite; ACTIVE_SPRITES],
    active_sprites_has_zero: bool,
    control: Control,
    status: Status,
    mask: Mask,
    // Sometimes called 'v', can hold address _or_ scrolling information.
    address: u16,
    // Sometimes called 't', can hold address _or_ scrolling information.
    temporary_address: u16,
    write_lower: bool,
    fine_x: u8,
    oam_address: u8,
    // Reading vblank just before it's set will prevent it being set and NMI being triggered
    suppress_vblank: bool,
}

impl<M: Memory> PPU<M> {
    pub fn with_memory(memory: M) -> Self {
        PPU {
            memory,
            read_buffer: 0,
            object_attribute_memory: [0; 256],
            scanline: 0,
            cycle_count: 0,
            tile_pattern: ShiftRegister::default(),
            palette_select: ShiftRegister::default(),
            active_sprites: [ActiveSprite::default(); ACTIVE_SPRITES],
            active_sprites_has_zero: false,
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            address: 0,
            temporary_address: 0,
            write_lower: false,
            fine_x: 0,
            oam_address: 0,
            suppress_vblank: false,
        }
    }

    fn address(&self) -> Address {
        Address::new(self.address)
    }

    fn set_address(&mut self, address: Address) {
        self.address = address.bytes();
    }

    fn scroll(&self) -> Scroll {
        Scroll::new(self.address)
    }

    fn set_scroll(&mut self, scroll: Scroll) {
        self.address = scroll.bits();
    }

    fn increment_address(&mut self) {
        self.address += self.control.address_increment();
    }

    fn tile_address(&self) -> Address {
        self.scroll().tile_address()
    }

    fn attribute_address(&self) -> Address {
        self.scroll().attribute_address()
    }

    fn increment_coarse_x(&mut self) {
        let mut scroll = self.scroll();
        scroll.increment_coarse_x();
        self.set_scroll(scroll);
    }

    fn increment_fine_y(&mut self) {
        let mut scroll = self.scroll();
        scroll.increment_fine_y();
        self.set_scroll(scroll);
    }

    fn transfer_horizontal_scroll(&mut self) {
        let mut scroll = self.scroll();
        let temporary_scroll = Scroll::new(self.temporary_address);
        scroll.set_horizontal(temporary_scroll);
        self.set_scroll(scroll);
    }

    fn load_sprites(&mut self) {
        if self.scanline == 0 {
            return;
        }

        let sprite_size = self.control.sprite_size();
        let table = self.control.sprite_pattern_table();

        let all_sprites = self.object_attribute_memory.chunks_exact(4).map(|chunk| {
            let attributes = SpriteAttributes::from_bits_truncate(chunk[2]);
            Sprite::new(chunk[3], chunk[0], chunk[1], attributes)
        });

        let scanline = self.scanline - 1;

        let sprites_on_scanline = all_sprites.enumerate().filter(|(_, sprite)| {
            let y = sprite.y as u16;
            scanline >= y && scanline < y + sprite_size.height() as u16
        });

        self.active_sprites = [ActiveSprite::default(); ACTIVE_SPRITES];
        self.active_sprites_has_zero = false;

        for (dest, (i, src)) in self.active_sprites.iter_mut().zip(sprites_on_scanline) {
            self.active_sprites_has_zero |= i == 0;
            *dest = ActiveSprite {
                sprite: src,
                ..Default::default()
            };
        }

        for i in 0..ACTIVE_SPRITES {
            let sprite = self.active_sprites[i].sprite;
            let attr = sprite.attributes;

            // Use wrapping_sub and % to handle default zero'd sprites at y = 0 without branching
            let y_in_sprite = scanline.wrapping_sub(sprite.y as u16) as u8 % sprite_size.height();
            let y_in_sprite = attr.ver_flip(y_in_sprite, sprite_size);

            let (sprite_table, index) = match sprite_size {
                SpriteSize::_8x8 => (table, sprite.tile_index),
                SpriteSize::_8x16 => (
                    PatternTable::from((sprite.tile_index & 0b1) == 1),
                    sprite.tile_index & 0b1111_1110,
                ),
            };

            let (pattern0, pattern1) = self.read_pattern_row(sprite_table, index, y_in_sprite);

            self.active_sprites[i].pattern0 = pattern0;
            self.active_sprites[i].pattern1 = pattern1;
        }
    }

    fn next_color(&mut self) -> Color {
        let sprite = self.sprite_color();
        let (background, background_opaque) = self.background_color();

        let color_address = if sprite.visible && sprite.priority {
            sprite.color_address
        } else if background_opaque {
            background
        } else if sprite.visible {
            sprite.color_address
        } else {
            background
        };

        if self.active_sprites_has_zero && sprite.index == 0 && background_opaque {
            self.status |= Status::SPRITE_ZERO_HIT;
        }

        Color(self.memory.read(color_address))
    }

    fn background_color(&self) -> (Address, bool) {
        let lower_bits = self.tile_pattern.get_bits(self.fine_x);
        let higher_bits = self.palette_select.get_bits(self.fine_x);

        let color_index = (lower_bits | (higher_bits << 2)) as u16;

        let show_background = self.mask.contains(Mask::SHOW_BACKGROUND);
        let opaque = show_background && lower_bits != 0;

        // Use universal background colour when transparent
        let color_address = BACKGROUND_PALETTES + color_index * opaque as u16;

        (color_address, opaque)
    }

    fn sprite_color(&self) -> SelectedSprite {
        let show_sprites = self.mask.contains(Mask::SHOW_SPRITES) && self.scanline > 0;

        // Bitflags for which sprites should be shown, to avoid branches
        let mut show: u8 = 0b0000_0000;
        // All 8 sprites, plus a 9th sprite that will always be 'None'
        let mut results: [(SpriteAttributes, u8); 9] = Default::default();

        for (index, active_sprite) in self.active_sprites.iter().enumerate() {
            let x = active_sprite.sprite.x as u16;
            let attr = active_sprite.sprite.attributes;

            // Use % to always handle default sprite with x = 0 without branching
            let x_in_sprite = attr.hor_flip(self.cycle_count.wrapping_sub(x) as u8 % 8);

            let bit0 = (active_sprite.pattern0 >> x_in_sprite) & 0b1;
            let bit1 = (active_sprite.pattern1 >> x_in_sprite) & 0b1;

            let lower_index = (bit1 << 1) | bit0;

            let transparent = lower_index == 0;

            let show_sprite = show_sprites
                && !transparent
                && self.cycle_count >= x
                && self.cycle_count < x + 8
                && self.scanline > active_sprite.sprite.y as u16;

            show |= (show_sprite as u8) << index;

            results[index] = (attr, lower_index);
        }

        // Find the highest-priority sprite that should be shown using trailing zeros in the bit flag
        let index = show.trailing_zeros() as usize;
        let (attr, lower_index) = results[index];

        let palette = (attr & SpriteAttributes::PALETTE).bits();
        let color_index = (palette << 2) | lower_index;
        let color_address = SPRITE_PALETTES + color_index.into();
        let priority = !attr.contains(SpriteAttributes::PRIORITY);

        SelectedSprite {
            visible: index < ACTIVE_SPRITES,
            color_address,
            priority,
            index,
        }
    }

    fn read_pattern_row(
        &mut self,
        nametable: PatternTable,
        pattern_index: u8,
        row: u8,
    ) -> (u8, u8) {
        debug_assert!(row < 16, "expected row < 16, but row = {}", row);
        // For 8x16 sprites, shift bit pattern to fetch lower tile:
        // row = 0b0000_abcd => 0b000a_0bcd
        let row = row & 0b0111 | ((row & 0b1000) << 1);

        let index = u16::from(pattern_index) << 4 | u16::from(row);
        let pattern_address0 = Address::from(nametable) + index;
        let pattern_address1 = pattern_address0 + 0b1000;

        let pattern0 = self.memory.read(pattern_address0);
        let pattern1 = self.memory.read(pattern_address1);

        (pattern0, pattern1)
    }

    fn read_next_tile(&mut self) {
        let scroll = self.scroll();
        let coarse_x = scroll.coarse_x();
        let coarse_y = scroll.coarse_y();
        let fine_y = scroll.fine_y();

        let pattern_index = self.memory.read(self.tile_address());
        let attribute_byte = self.memory.read(self.attribute_address());
        let attribute_bit_index0 = (coarse_y & 2) << 1 | (coarse_x & 2);
        let attribute_bit_index1 = attribute_bit_index0 + 1;

        let table = self.control.background_pattern_table();
        let (pattern0, pattern1) = self.read_pattern_row(table, pattern_index, fine_y);

        self.tile_pattern.set_next_bytes(pattern0, pattern1);

        let palette0 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index0);
        let palette1 = set_all_bits_to_bit_at_index(attribute_byte, attribute_bit_index1);

        self.palette_select.set_next_bytes(palette0, palette1);
    }

    fn rendering(&self) -> bool {
        self.mask
            .intersects(Mask::SHOW_BACKGROUND | Mask::SHOW_SPRITES)
    }

    pub fn tick(&mut self) -> PPUOutput {
        let mut interrupt = false;

        self.suppress_vblank = false;

        let in_bounds = self.scanline < 240 && self.cycle_count < 256;
        let rendering = self.rendering();

        match (self.scanline, self.cycle_count) {
            (_, 0) => self.load_sprites(),
            (241, 1) if !self.suppress_vblank => {
                // TODO: also suppress NMI the frame after, apparently
                self.status |= Status::VBLANK;

                if self.control.nmi_on_vblank() {
                    interrupt = true;
                }
            }
            (261, 1) => {
                // TODO: The VBLANK is much too long
                self.status -= Status::VBLANK | Status::SPRITE_ZERO_HIT;
                if rendering {
                    self.set_address(Address::new(self.temporary_address));
                }
            }
            (0..=239, 256) if rendering => self.increment_fine_y(),
            (0..=239, 257) if rendering => self.transfer_horizontal_scroll(),
            _ => {}
        }

        // TODO: not sure about these conditions
        // A tile is fetched every 8 cycles.
        // The 1st and 2nd tiles are fetched at the of the previous scanline, filling the 16-bit shift registers.
        // The first cycle is idle, so the 3rd tile is fetched at cycle 8.
        let preparing_next_scanline =
            (self.scanline < 240 || self.scanline == 261) && self.cycle_count >= 328;
        if rendering
            && ((in_bounds && self.cycle_count > 0) || preparing_next_scanline)
            && self.cycle_count % 8 == 0
        {
            self.read_next_tile();
            self.increment_coarse_x();
        }

        let color = in_bounds.then(|| self.next_color());

        // Don't shift registers in the last 4 bits, or everything goes out of alignment.
        // Oddly, the cycle count in a scanline isn't divisible by 8.
        if self.cycle_count < 336 {
            self.tile_pattern.shift();
            self.palette_select.shift();
        }

        let vblank = self.scanline >= 240;

        if self.cycle_count < 340 {
            self.cycle_count += 1;
        } else {
            self.cycle_count = 0;
            if self.scanline < 261 {
                self.scanline += 1;
            } else {
                self.scanline = 0;
            }
        };

        PPUOutput {
            color,
            interrupt,
            vblank,
        }
    }
}

impl<M: Debug> Debug for PPU<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PPU")
            .field("memory", &self.memory)
            .field("read_buffer", &self.read_buffer)
            .field("scanline", &self.scanline)
            .field("cycle_count", &self.cycle_count)
            .field("tile_pattern", &self.tile_pattern)
            .field("palette_select", &self.palette_select)
            .field("active_sprites", &self.active_sprites)
            .field("control", &self.control)
            .field("status", &self.status)
            .field("mask", &self.mask)
            .field("address", &self.address)
            .field("temporary_address", &self.temporary_address)
            .field("write_lower", &self.write_lower)
            .field("fine_x", &self.fine_x)
            .field("oam_address", &self.oam_address)
            .finish()
    }
}

pub struct PPUOutput {
    pub color: Option<Color>,
    pub interrupt: bool,
    /// vblank status sent to display, without quirks of the real PPU vblank
    pub vblank: bool,
}

#[derive(Default, Debug, Eq, PartialEq)]
struct ShiftRegister(u16, u16);

impl ShiftRegister {
    fn set_next_bytes(&mut self, byte0: u8, byte1: u8) {
        debug_assert_eq!(self.0 & 0x00FF, 0, "Lower byte should have shifted out");
        debug_assert_eq!(self.1 & 0x00FF, 0, "Lower byte should have shifted out");
        self.0 |= u16::from(byte0);
        self.1 |= u16::from(byte1);
    }

    fn shift(&mut self) {
        self.0 <<= 1;
        self.1 <<= 1;
    }

    fn get_bits(&self, bit: u8) -> u8 {
        debug_assert!(bit < 8);
        let bit0 = (self.0 >> (15 - bit)) & 0b1;
        let bit1 = (self.1 >> (14 - bit)) & 0b10;
        (bit0 | bit1) as u8
    }
}

#[derive(Copy, Clone)]
struct SelectedSprite {
    visible: bool,
    color_address: Address,
    priority: bool,
    index: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
struct ActiveSprite {
    sprite: Sprite,
    pattern0: u8,
    pattern1: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Sprite {
    x: u8,
    y: u8,
    tile_index: u8,
    attributes: SpriteAttributes,
}

impl Sprite {
    fn new(x: u8, y: u8, tile_index: u8, attributes: SpriteAttributes) -> Self {
        Sprite {
            x,
            y,
            tile_index,
            attributes,
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite::new(0xff, 0xff, 0xff, SpriteAttributes::all())
    }
}

bitflags! {
    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
    struct SpriteAttributes: u8 {
        const VERTICAL_FLIP   = 0b1000_0000;
        const HORIZONTAL_FLIP = 0b0100_0000;
        const PRIORITY        = 0b0010_0000;
        const PALETTE         = 0b0000_0011;
    }
}

impl SpriteAttributes {
    /// Bit-twiddling to avoid a conditional, same as `x = if hor_flip { x } else { 7 - x }`
    fn hor_flip(self, x: u8) -> u8 {
        x ^ (((!self & Self::HORIZONTAL_FLIP).bits() >> 6) * 0b0000_0111)
    }

    /// Bit-twiddling to avoid a conditional, same as `y = if ver_flip { height - 1 - y } else { y }`
    fn ver_flip(self, y: u8, sprite_size: SpriteSize) -> u8 {
        y ^ ((self.bits() >> 7) * (sprite_size.height() - 1))
    }
}

fn set_all_bits_to_bit_at_index(byte: u8, index: u8) -> u8 {
    (!((byte >> index) & 1)).wrapping_add(1)
}

impl<M: Memory> PPURegisters for PPU<M> {
    fn write_control(&mut self, byte: u8) {
        self.control = Control::from_bits(byte);

        // Set bits of temporary address to nametable
        self.temporary_address &= 0b1111_0011_1111_1111;
        self.temporary_address |= u16::from(self.control.nametable_select()) << 10;
    }

    fn write_mask(&mut self, byte: u8) {
        self.mask = Mask::from_bits_truncate(byte);
    }

    fn read_status(&mut self) -> u8 {
        self.write_lower = false;
        self.suppress_vblank = true;
        let bits = self.status.bits();
        self.status.remove(Status::VBLANK);
        bits
    }

    fn write_oam_address(&mut self, byte: u8) {
        self.oam_address = byte;
    }

    fn read_oam_data(&mut self) -> u8 {
        self.object_attribute_memory[self.oam_address as usize]
    }

    fn write_oam_data(&mut self, byte: u8) {
        self.object_attribute_memory[self.oam_address as usize] = byte;
        self.oam_address = self.oam_address.wrapping_add(1);
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
        if self.rendering() {
            // warn!("Attempt to write address to PPU during rendering");
        }
        if self.write_lower {
            self.temporary_address &= 0b1111_1111_0000_0000;
            self.temporary_address |= u16::from(byte);
            self.set_address(Address::new(self.temporary_address));
        } else {
            self.temporary_address &= 0b0000_0000_1111_1111;
            self.temporary_address |= u16::from(byte & 0b0011_1111) << 8;
        }

        self.write_lower = !self.write_lower;
    }

    fn read_data(&mut self) -> u8 {
        if cfg!(debug_assertions) && self.rendering() {
            warn!("Attempt to read from PPU during rendering");
        }
        let address = self.address();
        let byte = self.memory.read(address);
        self.increment_address();

        if address < BACKGROUND_PALETTES {
            let buffer = self.read_buffer;
            self.read_buffer = byte;
            buffer
        } else {
            byte
        }
    }

    fn write_data(&mut self, byte: u8) {
        if cfg!(debug_assertions) && self.rendering() {
            // warn!("Attempt to write to PPU during rendering");
        }
        self.memory.write(self.address(), byte);
        self.increment_address();
    }

    fn write_oam_dma(&mut self, mut bytes: [u8; 256]) {
        bytes.rotate_right(self.oam_address as usize);
        self.object_attribute_memory = bytes;
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Color(u8);

impl Color {
    pub fn to_byte(&self) -> u8 {
        self.0
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        COLOR_LOOKUP[self.0 as usize]
    }
}

const COLOR_LOOKUP: [(u8, u8, u8); 64] = [
    (0x54, 0x54, 0x54),
    (0x00, 0x1e, 0x74),
    (0x08, 0x10, 0x90),
    (0x30, 0x00, 0x88),
    (0x44, 0x00, 0x64),
    (0x5c, 0x00, 0x30),
    (0x54, 0x04, 0x00),
    (0x3c, 0x18, 0x00),
    (0x20, 0x2a, 0x00),
    (0x08, 0x3a, 0x00),
    (0x00, 0x40, 0x00),
    (0x00, 0x3c, 0x00),
    (0x00, 0x32, 0x3c),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x98, 0x96, 0x98),
    (0x08, 0x4c, 0xc4),
    (0x30, 0x32, 0xec),
    (0x5c, 0x1e, 0xe4),
    (0x88, 0x14, 0xb0),
    (0xa0, 0x14, 0x64),
    (0x98, 0x22, 0x20),
    (0x78, 0x3c, 0x00),
    (0x54, 0x5a, 0x00),
    (0x28, 0x72, 0x00),
    (0x08, 0x7c, 0x00),
    (0x00, 0x76, 0x28),
    (0x00, 0x66, 0x78),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xec, 0xee, 0xec),
    (0x4c, 0x9a, 0xec),
    (0x78, 0x7c, 0xec),
    (0xb0, 0x62, 0xec),
    (0xe4, 0x54, 0xec),
    (0xec, 0x58, 0xb4),
    (0xec, 0x6a, 0x64),
    (0xd4, 0x88, 0x20),
    (0xa0, 0xaa, 0x00),
    (0x74, 0xc4, 0x00),
    (0x4c, 0xd0, 0x20),
    (0x38, 0xcc, 0x6c),
    (0x38, 0xb4, 0xcc),
    (0x3c, 0x3c, 0x3c),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xec, 0xee, 0xec),
    (0xa8, 0xcc, 0xec),
    (0xbc, 0xbc, 0xec),
    (0xd4, 0xb2, 0xec),
    (0xec, 0xae, 0xec),
    (0xec, 0xae, 0xd4),
    (0xec, 0xb4, 0xb0),
    (0xe4, 0xc4, 0x90),
    (0xcc, 0xd2, 0x78),
    (0xb4, 0xde, 0x78),
    (0xa8, 0xe2, 0x90),
    (0x98, 0xe2, 0xb4),
    (0xa0, 0xd6, 0xe4),
    (0xa0, 0xa2, 0xa0),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
];

#[cfg(test)]
mod tests {
    use crate::mem;
    use crate::ppu::Sprite;
    use crate::Address;
    use crate::ArrayMemory;

    use super::*;

    #[test]
    fn each_tick_produces_a_color() {
        let memory = ArrayMemory::default();
        let mut ppu = PPU::with_memory(memory);
        let _color: Option<Color> = ppu.tick().color;
    }

    #[test]
    fn writing_ppu_control_sets_control() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_control(0b1010_1010);
        assert_eq!(ppu.control, Control::from_bits(0b1010_1010));
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
    fn writing_ppu_mask_sets_mask() {
        let mut ppu = PPU::with_memory(mem!());

        ppu.write_mask(0b1010_1010);
        assert_eq!(ppu.mask, Mask::from_bits_truncate(0b1010_1010));
    }

    #[test]
    fn reading_ppu_status_returns_status_and_clears_vblank() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.status |= Status::VBLANK;
        assert_eq!(ppu.read_status(), 0b1000_0000);
        assert!(!ppu.status.contains(Status::VBLANK));
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
        ppu.read_data(); // Dummy read due to internal buffer
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
    fn reading_ppu_data_reads_from_internal_buffer() {
        let mut ppu = PPU::with_memory(mem! {
            0x2000 => {
                0xAA, 0xBB, 0xCC, 0xDD
            }
        });

        ppu.write_address(0x20);
        ppu.write_address(0x00);

        ppu.read_data();
        assert_eq!(ppu.read_data(), 0xAA);
        assert_eq!(ppu.read_data(), 0xBB);

        ppu.write_address(0x20);
        ppu.write_address(0x00);

        assert_eq!(ppu.read_data(), 0xCC);
        assert_eq!(ppu.read_data(), 0xAA);
    }

    #[test]
    fn reading_ppu_data_from_palette_does_not_use_internal_buffer() {
        let mut ppu = PPU::with_memory(mem! {
            0x3f00 => {
                0xAA, 0xBB, 0xCC, 0xDD
            }
        });

        ppu.write_address(0x3f);
        ppu.write_address(0x00);

        assert_eq!(ppu.read_data(), 0xAA);
        assert_eq!(ppu.read_data(), 0xBB);
        assert_eq!(ppu.read_data(), 0xCC);

        ppu.write_address(0x3f);
        ppu.write_address(0x00);

        assert_eq!(ppu.read_data(), 0xAA);
        assert_eq!(ppu.read_data(), 0xBB);
    }

    #[test]
    fn reading_or_writing_ppu_data_increments_address_by_increment_in_control_register() {
        let mut ppu = PPU::with_memory(mem! {
            0x1234 => { 0x00, 0x64, 0x00, 0x74 }
            0x1254 => { 0x84 }
            0x1274 => { 0x00 }
            0x1294 => { 0x00 }
        });

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_control(0b0000_0000);

        ppu.write_data(0x54);
        ppu.read_data(); // Dummy read due to internal buffer
        assert_eq!(ppu.read_data(), 0x64);
        ppu.write_data(0x74);
        assert_eq!(ppu.memory.read(Address::new(0x1237)), 0x74);

        ppu.write_address(0x12);
        ppu.write_address(0x34);
        ppu.write_control(0b0000_0100);

        ppu.write_data(0x74);
        ppu.read_data(); // Dummy read due to internal buffer
        assert_eq!(ppu.read_data(), 0x84);
        ppu.write_data(0x94);
        assert_eq!(ppu.memory.read(Address::new(0x1294)), 0x94);
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
    fn writing_oam_dma_writes_from_oam_address_and_wraps() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.oam_address = 0x42;

        let mut data = [0; 256];

        for (i, elem) in data.iter_mut().enumerate() {
            *elem = i as u8;
        }

        ppu.write_oam_dma(data);

        data.rotate_right(0x42);
        assert_eq!(ppu.object_attribute_memory.to_vec(), data.to_vec());
    }

    #[test]
    fn reading_oam_data_reads_from_oam_address() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.oam_address = 0x42;
        ppu.object_attribute_memory[0x42] = 0x43;

        assert_eq!(ppu.read_oam_data(), 0x43);
    }

    #[test]
    fn writing_oam_data_writes_to_oam_address() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.oam_address = 0x42;

        ppu.write_oam_data(0x43);

        assert_eq!(ppu.object_attribute_memory[0x42], 0x43);
    }

    #[test]
    fn writing_oam_data_increments_oam_address() {
        let mut ppu = PPU::with_memory(mem!());
        ppu.oam_address = 0x42;

        ppu.write_oam_data(0x07);

        assert_eq!(ppu.oam_address, 0x43);
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

    #[test]
    fn loading_sprites_loads_all_sprites_for_a_given_scanline() {
        let mut ppu = PPU::with_memory(mem!());

        let oam = [
            // y, index, attributes, x
            [0, 0, 0, 0],
            [22, 1, 1, 1],
            [23, 2, 2, 2],
            [29, 3, 3, 3],
            [30, 4, 4, 4],
            [31, 5, 5, 5],
            [255, 6, 6, 6],
        ]
        .as_flattened();

        ppu.object_attribute_memory[..oam.len()].copy_from_slice(oam);

        ppu.scanline = 30;

        ppu.load_sprites();

        let expected = [
            ActiveSprite {
                sprite: Sprite::new(1, 22, 1, SpriteAttributes::from_bits_truncate(1)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(2, 23, 2, SpriteAttributes::from_bits_truncate(2)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(3, 29, 3, SpriteAttributes::from_bits_truncate(3)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite::default(),
            ActiveSprite::default(),
            ActiveSprite::default(),
            ActiveSprite::default(),
            ActiveSprite::default(),
        ];

        assert_eq!(ppu.active_sprites, expected);
    }

    #[test]
    fn when_more_than_eight_sprites_on_scanline_only_first_eight_are_loaded() {
        let mut ppu = PPU::with_memory(mem!());

        let oam = [
            // y, index, attributes, x
            23, 0, 0, 0, 23, 1, 1, 1, 24, 2, 2, 2, 24, 3, 3, 3, 25, 4, 4, 4, 25, 5, 5, 5, 26, 6, 6,
            6, 26, 7, 7, 7, 27, 8, 8, 8,
        ];

        ppu.object_attribute_memory[..oam.len()].copy_from_slice(&oam);

        ppu.scanline = 30;

        ppu.load_sprites();

        let expected = [
            ActiveSprite {
                sprite: Sprite::new(0, 23, 0, SpriteAttributes::from_bits_truncate(0)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(1, 23, 1, SpriteAttributes::from_bits_truncate(1)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(2, 24, 2, SpriteAttributes::from_bits_truncate(2)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(3, 24, 3, SpriteAttributes::from_bits_truncate(3)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(4, 25, 4, SpriteAttributes::from_bits_truncate(4)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(5, 25, 5, SpriteAttributes::from_bits_truncate(5)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(6, 26, 6, SpriteAttributes::from_bits_truncate(6)),
                pattern0: 0,
                pattern1: 0,
            },
            ActiveSprite {
                sprite: Sprite::new(7, 26, 7, SpriteAttributes::from_bits_truncate(7)),
                pattern0: 0,
                pattern1: 0,
            },
        ];

        assert_eq!(ppu.active_sprites, expected);
    }

    #[test]
    fn loading_sprites_clears_active_sprites() {
        let mut ppu = PPU::with_memory(mem!());

        let oam = [29, 3, 3, 3];
        ppu.object_attribute_memory[..oam.len()].copy_from_slice(&oam);

        ppu.scanline = 30;
        ppu.load_sprites();

        let cleared = [ActiveSprite::default(); 8];

        assert_ne!(ppu.active_sprites, cleared);

        ppu.scanline = 40;
        ppu.load_sprites();

        assert_eq!(ppu.active_sprites, cleared);
    }

    #[test]
    fn can_read_rows_from_nametable() {
        let mut ppu = PPU::with_memory(mem! {
            0x1050 => {
                // Bit 0
                0b0000_0001,
                0b0000_0010,
                0b0000_0100,
                0b0000_1000,
                0b0001_0000,
                0b0010_0000,
                0b0100_0000,
                0b1000_0000,
                // Bit 1
                0b1000_0000,
                0b0100_0000,
                0b0010_0000,
                0b0001_0000,
                0b0000_1000,
                0b0000_0100,
                0b0000_0010,
                0b0000_0001
            }
        });

        let table = PatternTable::Right;
        let index = 5;
        let row = 4;
        let (bit0, bit1) = ppu.read_pattern_row(table, index, row);

        assert_eq!(bit0, 0b0001_0000);
        assert_eq!(bit1, 0b0000_1000);
    }

    #[test]
    fn sprite_flips_coordinates() {
        let empty = SpriteAttributes::empty();
        let all = SpriteAttributes::all();
        let hor = SpriteAttributes::HORIZONTAL_FLIP;
        let ver = SpriteAttributes::VERTICAL_FLIP;
        for value in 0..8 {
            assert_eq!(empty.hor_flip(value), 7 - value);
            assert_eq!(empty.ver_flip(value, SpriteSize::_8x8), value);
            assert_eq!((all - hor).hor_flip(value), 7 - value);
            assert_eq!((all - ver).ver_flip(value, SpriteSize::_8x8), value);
            assert_eq!(hor.hor_flip(value), value);
            assert_eq!(ver.ver_flip(value, SpriteSize::_8x8), 7 - value);
            assert_eq!(all.hor_flip(value), value);
            assert_eq!(all.ver_flip(value, SpriteSize::_8x8), 7 - value);
        }

        for value in 0..16 {
            assert_eq!(empty.ver_flip(value, SpriteSize::_8x16), value);
            assert_eq!((all - ver).ver_flip(value, SpriteSize::_8x16), value);
            assert_eq!(ver.ver_flip(value, SpriteSize::_8x16), 15 - value);
            assert_eq!(all.ver_flip(value, SpriteSize::_8x16), 15 - value);
        }
    }
}
