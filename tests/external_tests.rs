use std::fs;
use std::io::Cursor;

use image::ColorType;

use nes_rust::INes;
use nes_rust::NoDisplay;
use nes_rust::NES;
use nes_rust::{Address, BufferDisplay, HEIGHT, WIDTH};

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
    blargg_test(
        "blargg_ppu_tests_palette_ram",
        include_bytes!("blargg_ppu_tests/palette_ram.nes"),
        0xe412,
    );
}

#[test]
fn blargg_ppu_tests_power_up_palette() {
    blargg_test(
        "blargg_ppu_tests_power_up_palette",
        include_bytes!("blargg_ppu_tests/power_up_palette.nes"),
        0xe3ac,
    );
}

#[test]
fn blargg_ppu_tests_sprite_ram() {
    blargg_test(
        "blargg_ppu_tests_sprite_ram",
        include_bytes!("blargg_ppu_tests/sprite_ram.nes"),
        0xe467,
    );
}

// TODO: PPU tick isn't right relative to CPU, cus we need to know ticks for each instruction type
#[test]
fn blargg_ppu_tests_vbl_clear_time() {
    blargg_test(
        "blargg_ppu_test_vbl_clear_time",
        include_bytes!("blargg_ppu_tests/vbl_clear_time.nes"),
        0xe3b3,
    );
}

// TODO
#[test]
#[ignore]
fn blargg_ppu_tests_vram_access() {
    blargg_test(
        "blargg_ppu_test_vram_access",
        include_bytes!("blargg_ppu_tests/vram_access.nes"),
        0xe48d,
    );
}

#[test]
fn blargg_cpu_timing_test() {
    blargg_test(
        "blargg_cpu_timing_test",
        include_bytes!("blargg_cpu_tests/cpu_timing_test.nes"),
        0xea5a,
    );
}

fn blargg_test(name: &str, test: &[u8], end_address: u16) {
    let _ = env_logger::builder().is_test(true).try_init();

    fs::create_dir_all("test_results").unwrap();
    let fname = format!("./test_results/{}_failure.png", name);
    let _ = fs::remove_file(&fname);

    let cursor = Cursor::new(test);

    let ines = INes::read(cursor).unwrap();
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge, BufferDisplay::new());

    const ITERATIONS: usize = 10_000_000;

    for _ in 0..ITERATIONS {
        if nes.program_counter() == Address::new(end_address) {
            let byte = nes.read_cpu(Address::new(0xf0));

            if byte == 0x01 {
                return;
            } else {
                let buffer = nes.display().buffer();
                fs::create_dir_all("test_results").unwrap();
                image::save_buffer(&fname, buffer, WIDTH.into(), HEIGHT.into(), ColorType::Rgb8)
                    .unwrap();
                panic!(
                    "Failed, error code: 0x{:02x}. Saved image in {}",
                    byte, fname
                )
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
        "Test didn't complete after {} iterations, last 3 program counters were: {} {} {}",
        ITERATIONS, pc1, pc2, pc3
    );
}
