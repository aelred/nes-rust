use std::io::Cursor;

use image::ColorType;

use nes_rust::{Address, BufferDisplay, HEIGHT, WIDTH};
use nes_rust::INes;
use nes_rust::NES;
use nes_rust::NoDisplay;

#[test]
fn nestest() {
    let _ = env_logger::builder().is_test(true).try_init();

    let cursor = Cursor::new(include_bytes!("nestest.nes"));

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

#[test]
fn blargg_ppu_tests_palette_ram() {
    blargg_ppu_test("palette_ram", include_bytes!("blargg_ppu_tests/palette_ram.nes"), 0xe412);
}

#[test]
fn blargg_ppu_tests_power_up_palette() {
    blargg_ppu_test("power_up_palette", include_bytes!("blargg_ppu_tests/power_up_palette.nes"), 0xe3ac);
}

#[test]
fn blargg_ppu_tests_sprite_ram() {
    blargg_ppu_test("sprite_ram", include_bytes!("blargg_ppu_tests/sprite_ram.nes"), 0xe467);
}

// TODO: PPU tick isn't right relative to CPU, cus we need to know ticks for each instruction type
#[test]
#[ignore]
fn blargg_ppu_tests_vbl_clear_time() {
    blargg_ppu_test("vbl_clear_time", include_bytes!("blargg_ppu_tests/vbl_clear_time.nes"), 0xe3b3);
}

// TODO
#[test]
#[ignore]
fn blargg_ppu_tests_vram_access() {
    blargg_ppu_test("vram_access", include_bytes!("blargg_ppu_tests/vram_access.nes"), 0xe48d);
}

fn blargg_ppu_test(name: &str, test: &[u8], end_address: u16) {
    let _ = env_logger::builder().is_test(true).try_init();

    let cursor = Cursor::new(test);

    let ines = INes::read(cursor).unwrap();
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge, BufferDisplay::new());

    for _ in 1..2_000_000 {
        if nes.program_counter() == Address::new(end_address) {
            let byte = nes.read_cpu(Address::new(0xf0));

            if byte == 0x01 {
                return;
            } else {
                let buffer = nes.display().buffer();
                let fname = format!("blarrg_ppu_test_{}_failure.png", name);
                image::save_buffer(&fname, buffer, WIDTH.into(), HEIGHT.into(), ColorType::Rgb8).unwrap();
                panic!("Failed, error code: 0x{:02x}. Saved image in {}", byte, fname)
            }
        }

        nes.tick();
    }

    let pc1 = nes.program_counter();
    nes.tick();
    let pc2 = nes.program_counter();
    nes.tick();
    let pc3 = nes.program_counter();

    panic!(
        "Test didn't complete after 2,000,000 iterations, last 3 program counters were: {} {} {}",
        pc1, pc2, pc3
    );
}