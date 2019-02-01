use std::io::Cursor;

use nes_rust::Address;
use nes_rust::INes;
use nes_rust::NoDisplay;
use nes_rust::NES;

const NESTEST: &[u8] = include_bytes!("nestest.nes");

#[test]
fn nestest() {
    env_logger::init();

    let cursor = Cursor::new(NESTEST);

    let ines = INes::read(cursor).unwrap();
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge, NoDisplay);

    nes.set_program_counter(Address::new(0xc000));

    loop {
        if nes.program_counter() == Address::new(0xc66e) {
            let byte0 = nes.read_cpu(Address::new(0x02));
            let byte1 = nes.read_cpu(Address::new(0x03));

            if (byte0, byte1) == (0, 0) {
                break;
            } else {
                panic!("Failed, error code: 0x{:02x}{:02x}", byte0, byte1)
            }
        }

        nes.tick();
    }
}
