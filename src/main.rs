use nes_rust::INes;
use nes_rust::INesReadError;
use nes_rust::NES;
use std::time::Duration;

fn main() -> Result<(), INesReadError> {
    env_logger::init();

    let stdin = std::io::stdin();
    let handle = stdin.lock();

    let ines = INes::read(handle)?;
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge);

    loop {
        nes.tick();
        std::thread::sleep(Duration::from_millis(100));
    }
}
