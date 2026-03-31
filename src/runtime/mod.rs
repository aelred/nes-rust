use anyhow::Result;
use std::time::Duration;

#[cfg(feature = "sdl")]
pub mod sdl;

#[cfg(feature = "web")]
mod web;

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

pub fn run(log_level: log::Level) -> Result<()> {
    ActiveRuntime::run(log_level)
}

trait Runtime {
    fn run(log_level: log::Level) -> Result<()>;
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
use web::Web as ActiveRuntime;

#[cfg(feature = "sdl")]
use sdl::Sdl as ActiveRuntime;

// No-op runtime when one isn't configured
#[cfg(not(any(feature = "web", feature = "sdl")))]
type ActiveRuntime = ();

impl Runtime for () {
    fn run(_log_level: log::Level) -> Result<()> {
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
