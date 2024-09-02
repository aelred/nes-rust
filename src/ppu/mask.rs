use bitflags::bitflags;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Mask(MaskFlags);

impl Mask {
    pub fn from_bits(bits: u8) -> Self {
        Self(MaskFlags::from_bits_truncate(bits))
    }

    pub fn show_background(self) -> bool {
        self.0.contains(MaskFlags::SHOW_BACKGROUND)
    }

    pub fn show_sprites(self) -> bool {
        self.0.contains(MaskFlags::SHOW_SPRITES)
    }
}

bitflags! {
    #[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
    struct MaskFlags: u8 {
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
