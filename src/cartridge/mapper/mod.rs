mod mmc1;
mod nrom;
mod uxrom;

pub use crate::cartridge::mapper::mmc1::MMC1;
pub use crate::cartridge::mapper::nrom::NROM;
pub use crate::cartridge::mapper::uxrom::UxROM;
use crate::cartridge::NametableMirroring;
use crate::Address;
use enum_dispatch::enum_dispatch;
use std::fmt::Debug;

#[enum_dispatch]
pub trait Mapper: Debug {
    /// Map a CPU address to an address in the PRG
    fn map(&self, address: Address) -> PRGAddress;

    #[allow(unused)]
    /// Map a CPU write to a register, if one exists.
    /// Return true if a register was written.
    fn write_register(&self, address: Address, data: u8) -> bool {
        false
    }

    // TODO: mirroring should be configurable, perhaps not even stored here
    /// Return the current nametable mirroring configuration
    fn nametable_mirroring(&self) -> NametableMirroring {
        NametableMirroring::default()
    }
}

#[enum_dispatch(Mapper)]
#[derive(Debug)]
pub enum AnyMapper {
    NROM,
    MMC1,
    UxROM,
}

pub enum PRGAddress {
    ROM(usize),
    RAM(usize),
    Unmapped,
}
