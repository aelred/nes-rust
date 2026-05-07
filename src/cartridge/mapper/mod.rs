pub mod mmc1;
pub mod nrom;
pub mod uxrom;

use crate::cartridge::NametableMirroring;
use crate::Address;
use std::fmt::Debug;

pub trait Mapper: Debug + Send {
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

pub enum PRGAddress {
    ROM(usize),
    RAM(usize),
    Unmapped,
}
