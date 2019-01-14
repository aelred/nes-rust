use std::io::Read;
use std::io;
use enum_primitive_derive::Primitive;
use num_traits::cast::FromPrimitive;

const PRG_ROM_SIZE_LOCATION: usize = 4;
const MAPPER_LOW_LOCATION: usize = 6;
const MAPPER_HIGH_LOCATION: usize = 7;

const _16KB: usize = 16_384;

#[derive(Debug)]
enum INesReadError {
    IO(io::Error),
    UnrecognisedMapper(u8),
}

impl From<io::Error> for INesReadError {
    fn from(error: io::Error) -> Self {
        INesReadError::IO(error)
    }
}

struct INes {
    prg_rom: Box<[u8]>,
    mapper: Mapper,
}

impl INes {
    fn read<R: Read>(mut reader: R) -> Result<Self, INesReadError> {
        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;

        let mapper = INes::mapper(header)?;

        let prg_rom_size = header[PRG_ROM_SIZE_LOCATION];
        let mut prg_rom = vec![0u8; prg_rom_size as usize * _16KB];
        reader.read_exact(prg_rom.as_mut())?;

        let ines = INes {
            prg_rom: prg_rom.into_boxed_slice(),
            mapper,
        };

        Ok(ines)
    }

    fn mapper(header: [u8; 16]) -> Result<Mapper, INesReadError> {
        let low = header[MAPPER_LOW_LOCATION] >> 4;
        let high = header[MAPPER_HIGH_LOCATION] & 0b1111_0000;
        let byte = low | high;
        Mapper::from_u8(byte).ok_or(INesReadError::UnrecognisedMapper(byte))
    }
}

#[derive(Debug, Eq, PartialEq, Primitive)]
enum Mapper {
    NROM = 0,
    Namco129 = 19
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_read_prg_rom_data_from_ines_file() {
        const SIZE: u8 = 10;
        let header: [u8; 16] = [0x4E, 0x45, 0x53, 0x1A, SIZE, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        // 160kb of prg data
        let mut prg_rom_data = vec![0; 163840];

        for i in 0..163840 {
            prg_rom_data[i] = i as u8;
        }

        let cursor = Cursor::new(header).chain(Cursor::new(prg_rom_data.clone()));

        let ines = INes::read(cursor).unwrap();

        assert_eq!(ines.prg_rom.len(), 163840);
        assert_eq!(Vec::from(ines.prg_rom), prg_rom_data);
    }

    #[test]
    fn can_read_mapper_from_ines_file() {
        // this maps to mapper 19
        let low: u8 = 0b0011_0000;
        let high: u8 = 0b0001_0000;

        let header: [u8; 16] = [0x4E, 0x45, 0x53, 0x1A, 2, 1, low, high, 0, 0, 0, 0, 0, 0, 0, 0];

        let cursor = Cursor::new(header).chain(std::io::repeat(0));

        let ines = INes::read(cursor).unwrap();

        assert_eq!(ines.mapper, Mapper::Namco129);
    }
}