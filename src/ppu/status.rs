use bitflags::bitflags;

bitflags! {
    #[derive(Default, Copy, Clone, Debug)]
    pub struct Status: u8 {
        const VBLANK          = 0b1000_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}
