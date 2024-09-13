use std::fs;
use std::io::Cursor;

use image::ColorType;

use nes_rust::INes;
use nes_rust::NES;
use nes_rust::{Address, BufferDisplay, HEIGHT, WIDTH};
use yare::parameterized;

enum Setup {
    Default,
    ProgramCounter(u16),
}

enum Terminate {
    #[allow(dead_code)]
    // Useful for debugging or adding new tests
    Never,
    Address(u16),
}

enum Success {
    Screen(&'static [u8]),
    Byte(u16, u8),
    Short(u16, u16),
}

#[parameterized(
    nestest = {
        "nestest", include_bytes!("nestest/nestest.nes"),
        Setup::ProgramCounter(0xc000), Terminate::Address(0xc66e), Success::Short(0x02, 0x0000)
    },
    blargg_ppu_tests_palette_ram = {
        "blargg_ppu_tests_palette_ram", include_bytes!("blargg_ppu_tests/palette_ram.nes"),
        Setup::Default, Terminate::Address(0xe412), Success::Byte(0xf0, 0x01)
    },
    blargg_ppu_tests_power_up_palette = {
        "blargg_ppu_tests_power_up_palette", include_bytes!("blargg_ppu_tests/power_up_palette.nes"),
        Setup::Default, Terminate::Address(0xe3ac), Success::Byte(0xf0, 0x01)
    },
    blargg_ppu_tests_sprite_ram = {
        "blargg_ppu_tests_sprite_ram", include_bytes!("blargg_ppu_tests/sprite_ram.nes"),
        Setup::Default, Terminate::Address(0xe467), Success::Byte(0xf0, 0x01)
    },
    // TODO: PPU tick isn't right relative to CPU, cus we need to know ticks for each instruction type
    blargg_ppu_tests_vbl_clear_time = {
        "blargg_ppu_test_vbl_clear_time", include_bytes!("blargg_ppu_tests/vbl_clear_time.nes"),
        Setup::Default, Terminate::Address(0xe3b3), Success::Byte(0xf0, 0x01)
    },
    // TODO
    // blargg_ppu_tests_vram_access = {
    //     "blargg_ppu_test_vram_access", include_bytes!("blargg_ppu_tests/vram_access.nes"),
    //     Setup::Default, Terminate::Address(0xe48d), Success::Byte(0xf0, 0x01)
    // },
    blargg_cpu_timing_test = {
        "blargg_cpu_timing_test", include_bytes!("blargg_cpu_tests/cpu_timing_test.nes"),
        Setup::Default, Terminate::Address(0xea5a), Success::Screen(include_bytes!("blargg_cpu_tests/success_screen.png"))
    },
    vbl_basics = {
        "vbl_basics", include_bytes!("ppu_vbl_nmi/rom_singles/01-vbl_basics.nes"),
        Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    },
    // TODO
    vbl_set_time = {
        "vbl_set_time", include_bytes!("ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes"),
        Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    },
    vbl_clear_time = {
        "vbl_clear_time", include_bytes!("ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes"),
        Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    },
    // TODO
    // nmi_control = {
    //     "nmi_control", include_bytes!("ppu_vbl_nmi/rom_singles/04-nmi_control.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // nmi_timing = {
    //     "nmi_timing", include_bytes!("ppu_vbl_nmi/rom_singles/05-nmi_timing.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // suppression = {
    //     "suppression", include_bytes!("ppu_vbl_nmi/rom_singles/06-suppression.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // nmi_on_timing = {
    //     "nmi_on_timing", include_bytes!("ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // nmi_off_timing = {
    //     "nmi_off_timing", include_bytes!("ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // even_odd_frames = {
    //     "even_odd_frames", include_bytes!("ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes"),
    //     Setup::Default, Terminate::Address(0xe8d5), Success::Byte(0x6000, 0x00)
    // },
    // TODO
    // even_odd_timing = {
    //     "even_odd_timing", include_bytes!("ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes"),
    //     Setup::Default, Terminate::Address(0xead5), Success::Byte(0x6000, 0x00)
    // },
)]
fn external_test(
    name: &str,
    test: &[u8],
    setup: Setup,
    terminate_check: Terminate,
    success_check: Success,
) {
    let _ = env_logger::builder().is_test(true).try_init();
    clear_nes_test_result_image(name);

    let cursor = Cursor::new(test);
    let ines = INes::read(cursor).unwrap();
    let cartridge = ines.into_cartridge();

    let mut nes = NES::new(cartridge, BufferDisplay::default());

    match setup {
        Setup::Default => {}
        Setup::ProgramCounter(address) => nes.set_program_counter(Address::new(address)),
    }

    const ITERATIONS: usize = 10_000_000;

    for cycles in 0..ITERATIONS {
        let terminated = match terminate_check {
            Terminate::Never => false,
            Terminate::Address(address) => nes.program_counter() == Address::new(address),
        };

        if !terminated {
            nes.tick();
            continue;
        }

        match get_result(success_check, &mut nes) {
            Ok(()) => {
                if cycles < 10 {
                    panic!("Test passed suspiciously quickly, only {} cycles", cycles);
                }
                return;
            }
            Err(message) => {
                let fname = save_nes_test_result_image(name, &nes);
                panic!("Failed: {}. Saved image in {}", message, fname)
            }
        }
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

fn get_result(success_check: Success, nes: &mut NES<BufferDisplay>) -> Result<(), String> {
    match success_check {
        Success::Screen(bytes) => {
            let success_screen = image::load_from_memory(bytes).unwrap();
            if success_screen.as_bytes() == nes.display().buffer() {
                Ok(())
            } else {
                Err("Screen doesn't match success".to_owned())
            }
        }
        Success::Byte(address, expected) => {
            let result = nes.read_cpu(Address::new(address));
            if result == expected {
                Ok(())
            } else {
                Err(format!("Expected 0x{:02x}, got 0x{:02x}", expected, result))
            }
        }
        Success::Short(address, expected) => {
            let byte1 = nes.read_cpu(Address::new(address));
            let byte2 = nes.read_cpu(Address::new(address + 1));
            let result = byte1 as u16 | (byte2 as u16) << 8;
            if result == expected {
                Ok(())
            } else {
                Err(format!("Expected 0x{:04x}, got 0x{:04x}", expected, result))
            }
        }
    }
}

fn clear_nes_test_result_image(name: &str) {
    let fname = nes_test_result_image_name(name);
    fs::create_dir_all("test_results").unwrap();
    let _ = fs::remove_file(&fname);
}

fn save_nes_test_result_image(name: &str, nes: &NES<BufferDisplay>) -> String {
    let fname = nes_test_result_image_name(name);
    let buffer = nes.display().buffer();
    image::save_buffer(&fname, buffer, WIDTH.into(), HEIGHT.into(), ColorType::Rgb8).unwrap();
    fname
}

fn nes_test_result_image_name(name: &str) -> String {
    format!("./test_results/{}_failure.png", name)
}
