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
