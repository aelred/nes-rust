use anyhow::{anyhow, bail, Result};
use std::fs::File;
use std::thread;
use std::time::{Duration, Instant};

use super::Runtime;
use crate::audio::{AudioSource, AUDIO_SAMPLE_SIZE, TARGET_AUDIO_FREQ};
use crate::runner::NESRunner;
use crate::video::FrontBuffer;
use crate::INes;
use crate::{Buttons, HEIGHT, WIDTH};
use sdl2::audio::AudioCallback;
use sdl2::audio::AudioDevice;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::WindowCanvas;

const SCALE: u16 = 3;
const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

type AudioListener = Box<dyn FnMut(&[f32]) + Send>;

pub struct SdlParams {
    pub log_level: log::Level,
    pub audio_listener: AudioListener,
}

impl Default for SdlParams {
    fn default() -> Self {
        Self {
            log_level: log::Level::Info,
            audio_listener: Box::new(|_| {}),
        }
    }
}

pub fn run_with(params: SdlParams) -> Result<()> {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(params.log_level.to_level_filter())
        .init();

    let sdl_context = sdl2::init().anyhow()?;

    let mut event_pump = sdl_context.event_pump().anyhow()?;

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

    let (mut runner, front_buffer, audio_source) = NESRunner::new();

    let mut display = SDLDisplay::new(&sdl_context, front_buffer)?;
    let _speaker = SDLSpeaker::new(&sdl_context, audio_source, params.audio_listener)?;

    runner.load_cartridge(cartridge);
    runner.resume();

    let mut expected_time = Duration::ZERO;
    let start = Instant::now();

    loop {
        expected_time += FRAME_DURATION;
        let actual_time = start.elapsed();
        if actual_time < expected_time {
            thread::sleep(expected_time - actual_time);
        }

        display.present()?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(());
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    runner.press(keycode_binding(key));
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    runner.release(keycode_binding(key));
                }
                _ => {}
            }
        }

        if runner.stopped() {
            bail!("NES stopped");
        }
    }
}

pub struct Sdl;

impl Runtime for Sdl {
    fn run(log_level: log::Level) -> Result<()> {
        run_with(SdlParams {
            log_level,
            ..Default::default()
        })
    }
}

fn keycode_binding(keycode: Keycode) -> Buttons {
    match keycode {
        Keycode::Z | Keycode::A => Buttons::A,
        Keycode::X | Keycode::S => Buttons::B,
        Keycode::RShift | Keycode::LShift => Buttons::SELECT,
        Keycode::Return => Buttons::START,
        Keycode::Up => Buttons::UP,
        Keycode::Down => Buttons::DOWN,
        Keycode::Left => Buttons::LEFT,
        Keycode::Right => Buttons::RIGHT,
        _ => Buttons::empty(),
    }
}

struct SDLDisplay {
    canvas: WindowCanvas,
    texture: Texture,
    buffer: FrontBuffer,
}

impl SDLDisplay {
    fn new(sdl_context: &sdl2::Sdl, buffer: FrontBuffer) -> Result<Self> {
        let video = sdl_context.video().anyhow()?;

        let window = video
            .window("nes-rust", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().target_texture().build()?;

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.set_scale(SCALE as f32, SCALE as f32).anyhow()?;
        canvas.clear();
        canvas.present();

        let creator = canvas.texture_creator();
        let format = PixelFormatEnum::RGBA32;
        let texture = creator.create_texture_streaming(format, WIDTH as u32, HEIGHT as u32)?;

        Ok(Self {
            canvas,
            texture,
            buffer,
        })
    }

    fn present(&mut self) -> Result<()> {
        let buffer = self.buffer.read_buffer();
        let pitch = WIDTH as usize * 4;
        self.texture.update(None, buffer, pitch)?;
        self.canvas.copy(&self.texture, None, None).anyhow()?;
        self.canvas.present();
        Ok(())
    }
}

struct SDLSpeaker {
    _device: AudioDevice<SDLCallback>,
}

impl SDLSpeaker {
    fn new(
        sdl_context: &sdl2::Sdl,
        audio_source: AudioSource,
        listener: AudioListener,
    ) -> Result<Self> {
        let audio = sdl_context.audio().anyhow()?;

        let spec = AudioSpecDesired {
            freq: Some(TARGET_AUDIO_FREQ as i32),
            channels: Some(1),
            samples: Some(AUDIO_SAMPLE_SIZE as u16),
        };

        let callback = SDLCallback {
            audio_source,
            listener,
        };

        let device = audio.open_playback(None, &spec, |_| callback).anyhow()?;
        device.resume();

        Ok(Self { _device: device })
    }
}

struct SDLCallback {
    audio_source: AudioSource,
    listener: AudioListener,
}

impl AudioCallback for SDLCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.audio_source.read(out);
        (self.listener)(out);
    }
}

trait SdlResult<T> {
    fn anyhow(self) -> Result<T>;
}

impl<T> SdlResult<T> for Result<T, String> {
    fn anyhow(self) -> Result<T> {
        self.map_err(|s| anyhow!(s))
    }
}
