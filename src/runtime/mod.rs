use std::{error::Error, time::Duration};

#[cfg(feature = "sdl")]
mod sdl;

#[cfg(feature = "web")]
mod web;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use web::Web as ActiveRuntime;

#[cfg(feature = "sdl")]
pub use sdl::Sdl as ActiveRuntime;

pub trait Runtime {
    fn init_log(level: log::Level) -> Result<(), Box<dyn Error>>;
    fn run() -> Result<(), Box<dyn Error>>;
}

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);
// True frequency is 1789773Hz, but tuned to match my emulator's rate
const NES_AUDIO_FREQ: f64 = 1_866_000.0;
const TARGET_AUDIO_FREQ: i32 = 44100;

// No-op runtime when one isn't configured
#[cfg(not(any(feature = "web", feature = "sdl")))]
pub type ActiveRuntime = ();

impl Runtime for () {
    fn init_log(_level: log::Level) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn run() -> Result<(), Box<dyn Error>> {
        use crate::{INes, NES};
        use std::fs::File;

        let args: Vec<String> = std::env::args().collect();

        let ines = if let Some(filename) = args.get(1) {
            let file = File::open(filename)?;
            INes::read(file)?
        } else {
            let stdin = std::io::stdin();
            let handle = stdin.lock();
            INes::read(handle)?
        };

        let cartridge = ines.into_cartridge();

        let mut nes = NES::new(cartridge, (), ());
        // TODO: maybe execute indefinitely?
        nes.tick();
        Ok(())
    }
}
