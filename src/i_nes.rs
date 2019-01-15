use crate::cartridge::Cartridge;
use crate::mapper::Mapper;
use num_traits::cast::FromPrimitive;
use std::io;
use std::io::Read;

const PRG_ROM_SIZE_LOCATION: usize = 4;
const CHR_ROM_SIZE_LOCATION: usize = 5;
const MAPPER_LOW_LOCATION: usize = 6;
const MAPPER_HIGH_LOCATION: usize = 7;

const _8KB: usize = 8_192;
const _16KB: usize = 16_384;

#[derive(Debug)]
pub enum INesReadError {
    IO(io::Error),
    UnrecognisedMapper(u8),
}

impl From<io::Error> for INesReadError {
    fn from(error: io::Error) -> Self {
        INesReadError::IO(error)
    }
}

pub struct INes {
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    mapper: Mapper,
}

impl INes {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, INesReadError> {
        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;

        let mapper = INes::mapper(header)?;

        let prg_rom_size = header[PRG_ROM_SIZE_LOCATION];
        let mut prg_rom = vec![0u8; prg_rom_size as usize * _16KB];
        reader.read_exact(prg_rom.as_mut())?;

        let chr_rom_size = header[CHR_ROM_SIZE_LOCATION];
        let mut chr_rom = vec![0u8; chr_rom_size as usize * _8KB];
        reader.read_exact(chr_rom.as_mut())?;

        let ines = INes {
            prg_rom: prg_rom.into_boxed_slice(),
            chr_rom: chr_rom.into_boxed_slice(),
            mapper,
        };

        Ok(ines)
    }

    pub fn into_cartridge(self) -> Cartridge {
        Cartridge::new(self.prg_rom, self.chr_rom, self.mapper)
    }

    fn mapper(header: [u8; 16]) -> Result<Mapper, INesReadError> {
        let low = header[MAPPER_LOW_LOCATION] >> 4;
        let high = header[MAPPER_HIGH_LOCATION] & 0b1111_0000;
        let byte = low | high;
        Mapper::from_u8(byte).ok_or_else(|| INesReadError::UnrecognisedMapper(byte))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_read_prg_rom_data_from_ines_file() {
        const SIZE: u8 = 10;
        let header: [u8; 16] = [
            0x4E, 0x45, 0x53, 0x1A, SIZE, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        // 160kb of prg data
        let mut prg_rom_data = vec![0; 163_840];

        for (i, item) in prg_rom_data.iter_mut().enumerate() {
            *item = i as u8;
        }

        let chr_rom_data = vec![0; 8_1920];

        let prg_cursor = Cursor::new(prg_rom_data.clone());
        let chr_cursor = Cursor::new(chr_rom_data.clone());
        let cursor = Cursor::new(header).chain(prg_cursor).chain(chr_cursor);

        let ines = INes::read(cursor).unwrap();

        assert_eq!(ines.prg_rom.len(), 163_840);
        assert_eq!(Vec::from(ines.prg_rom), prg_rom_data);
    }

    #[test]
    fn can_read_chr_rom_data_from_ines_file() {
        const SIZE: u8 = 10;
        let header: [u8; 16] = [
            0x4E, 0x45, 0x53, 0x1A, 1, SIZE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let prg_rom_data = vec![0; 16_384];

        // 80kb of chr data
        let mut chr_rom_data = vec![0; 81_920];

        for (i, item) in chr_rom_data.iter_mut().enumerate() {
            *item = i as u8;
        }

        let prg_cursor = Cursor::new(prg_rom_data.clone());
        let chr_cursor = Cursor::new(chr_rom_data.clone());
        let cursor = Cursor::new(header).chain(prg_cursor).chain(chr_cursor);

        let ines = INes::read(cursor).unwrap();

        assert_eq!(ines.chr_rom.len(), 81_920);
        assert_eq!(Vec::from(ines.chr_rom), chr_rom_data);
    }

    #[test]
    fn can_read_mapper_from_ines_file() {
        // this maps to mapper 19
        let low: u8 = 0b0011_0000;
        let high: u8 = 0b0001_0000;

        let header: [u8; 16] = [
            0x4E, 0x45, 0x53, 0x1A, 2, 1, low, high, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let cursor = Cursor::new(header).chain(std::io::repeat(0));

        let ines = INes::read(cursor).unwrap();

        assert_eq!(ines.mapper, Mapper::Namco129);
    }
}
