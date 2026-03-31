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
        panic!("No runtime configured, make sure target and features are set correctly")
    }
}
