use std::error::Error;

#[cfg(feature = "sdl")]
mod sdl;

#[cfg(feature = "web")]
mod web;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use web::Web as ActiveRuntime;

#[cfg(feature = "sdl")]
pub use sdl::Sdl as ActiveRuntime;

pub trait Runtime {
    fn run() -> Result<(), Box<dyn Error>>;
}
