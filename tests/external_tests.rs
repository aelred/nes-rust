use nes_rust::INes;
use nes_rust::NES;
use std::time::Duration;
use std::io::Cursor;
use nes_rust::Address;

const NESTEST: &'static [u8] = include_bytes!("nestest.nes");

#[test]
fn nestest() {
    env_logger::init();

    let cursor = Cursor::new(NESTEST);

    let ines = INes::read(cursor).unwrap();
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge);

    nes.set_program_counter(Address::new(0xc000));

    loop {
        let byte0 = nes.read_cpu(Address::new(0x02));
        let byte1 = nes.read_cpu(Address::new(0x03));

        match &[byte0, byte1] {
            &[0, 0] => {}
            b"OK" => {
                break
            },
            _ => {
                panic!("Failed, error code: {}{}", byte0, byte1)
            }
        }

        nes.tick();
    }
}