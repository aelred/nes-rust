use crate::INesReadError;

#[derive(Debug, Eq, PartialEq)]
pub enum Mapper {
    NROM,
    MMC1,
    Namco129,
}

impl TryFrom<u8> for Mapper {
    type Error = INesReadError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::NROM,
            1 => Self::MMC1,
            19 => Self::Namco129,
            _ => return Err(Self::Error::UnrecognisedMapper(value)),
        })
    }
}
