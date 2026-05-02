use anyhow::{bail, Result};

#[derive(Debug, Eq, PartialEq)]
pub enum Mapper {
    NROM,
    UxROM,
    MMC1,
    Namco129,
}

impl TryFrom<u8> for Mapper {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            0 => Self::NROM,
            1 => Self::MMC1,
            2 => Self::UxROM,
            19 => Self::Namco129,
            _ => bail!("Unrecognised mapper: {value}"),
        })
    }
}
