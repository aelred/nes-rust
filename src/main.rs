use nes_rust::INes;
use nes_rust::NES;
use nes_rust::INesReadError;

fn main() -> Result<(), INesReadError> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let ines = INes::read(handle)?;
    let cartridge = ines.into_cartridge();

    let mut nes = NES::new(cartridge);

    loop {
        nes.tick();
    }
}
