use bitflags::bitflags;

use crate::Address;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Control(ControlFlags);

impl Control {
    pub fn from_bits(bits: u8) -> Self {
        Self(ControlFlags::from_bits_truncate(bits))
    }

    pub fn nametable_select(self) -> u8 {
        (self.0 & ControlFlags::NAMETABLE_SELECT).bits()
    }

    pub fn background_pattern_table_address(self) -> Address {
        // 0x1000 if flag is set, 0x0000 otherwise
        Address::new(u16::from((self.0 & ControlFlags::BACKGROUND_PATTERN_TABLE).bits()) << 8)
    }

    pub fn sprite_pattern_table_address(self) -> Address {
        // 0x1000 if flag is set, 0x0000 otherwise
        Address::new(u16::from((self.0 & ControlFlags::SPRITE_PATTERN_TABLE).bits()) << 9)
    }

    pub fn address_increment(self) -> u16 {
        let set_case = (self.0 & ControlFlags::ADDRESS_INCREMENT).bits() << 3;
        let unset_case = (!self.0 & ControlFlags::ADDRESS_INCREMENT).bits() >> 2;
        (set_case | unset_case).into()
    }

    pub fn nmi_on_vblank(self) -> bool {
        self.0.contains(ControlFlags::NMI_ON_VBLANK)
    }
}

bitflags! {
    #[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
    struct ControlFlags: u8 {
        const NMI_ON_VBLANK            = 0b1000_0000;
        const PPU_MASTER_SLAVE         = 0b0100_0000;
        const SPRITE_SIZE              = 0b0010_0000;
        const BACKGROUND_PATTERN_TABLE = 0b0001_0000;
        const SPRITE_PATTERN_TABLE     = 0b0000_1000;
        const ADDRESS_INCREMENT        = 0b0000_0100;
        const NAMETABLE_SELECT         = 0b0000_0011;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_specifies_background_pattern_table_address() {
        let control = Control::from_bits(0b0000_0000);
        assert_eq!(
            control.background_pattern_table_address(),
            Address::new(0x0000)
        );
        let control = Control::from_bits(0b0001_0000);
        assert_eq!(
            control.background_pattern_table_address(),
            Address::new(0x1000)
        );
    }

    #[test]
    fn control_specifies_sprite_pattern_table_address() {
        let control = Control::from_bits(0b0000_0000);
        assert_eq!(control.sprite_pattern_table_address(), Address::new(0x0000));
        let control = Control::from_bits(0b0000_1000);
        assert_eq!(control.sprite_pattern_table_address(), Address::new(0x1000));
    }

    #[test]
    fn control_specifies_address_increment() {
        let control = Control::from_bits(0b0000_0000);
        assert_eq!(control.address_increment(), 1);
        let control = Control::from_bits(0b0000_0100);
        assert_eq!(control.address_increment(), 32);
    }

    #[test]
    fn control_specifies_nmi_on_vblank() {
        let control = Control::from_bits(0b0000_0000);
        assert_eq!(control.nmi_on_vblank(), false);
        let control = Control::from_bits(0b1000_0000);
        assert_eq!(control.nmi_on_vblank(), true);
    }
}
