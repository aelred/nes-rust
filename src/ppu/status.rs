use bitflags::bitflags;
use log::trace;

#[derive(Default, Copy, Clone, Debug)]
pub struct Status(StatusFlags);

impl Status {
    pub fn read(&mut self) -> u8 {
        let bits = self.0.bits();
        self.0.remove(StatusFlags::VBLANK);
        bits
    }

    pub fn vblank(self) -> bool {
        self.0.contains(StatusFlags::VBLANK)
    }

    pub fn enter_vblank(&mut self) {
        trace!("Entering vblank");
        self.0.insert(StatusFlags::VBLANK);
    }

    pub fn exit_vblank(&mut self) {
        trace!("Exiting vblank");
        self.0.remove(StatusFlags::VBLANK);
    }

    pub fn sprite_zero_hit(&mut self) {
        self.0.insert(StatusFlags::SPRITE_ZERO_HIT);
    }

    pub fn sprite_zero_clear(&mut self) {
        self.0.remove(StatusFlags::SPRITE_ZERO_HIT);
    }
}

bitflags! {
    #[derive(Default, Copy, Clone, Debug)]
    struct StatusFlags: u8 {
        const VBLANK          = 0b1000_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_status_resets_vblank() {
        let mut status = Status(StatusFlags::from_bits_truncate(0b1010_0000));

        assert!(status.vblank());
        assert_eq!(status.read(), 0b1010_0000);
        assert!(!status.vblank());
        assert_eq!(status.read(), 0b0010_0000);
    }
}
