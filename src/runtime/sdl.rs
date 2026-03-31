use anyhow::{anyhow, Result};
use std::fs::File;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::{Runtime, FRAME_DURATION};
use crate::audio::{audio_pipeline, AudioSink, AudioSource, AUDIO_SAMPLE_SIZE, TARGET_AUDIO_FREQ};
use crate::display_triple_buffer;
use crate::{Buttons, HEIGHT, WIDTH};
use crate::{Command, NES};
use crate::{FrontBuffer, INes};
use sdl2::audio::AudioCallback;
use sdl2::audio::AudioDevice;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::WindowCanvas;

const SCALE: u16 = 3;

pub struct Sdl;

impl Sdl {
    pub fn run_with(sdl_context: &sdl2::Sdl, speaker: AudioSink) -> Result<()> {
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

        let (front_buffer, back_buffer) = display_triple_buffer();

        let mut display = SDLDisplay::new(sdl_context, front_buffer)?;

        let mut nes = NES::new(cartridge, back_buffer, speaker);
        let mut expected_time = Duration::ZERO;
        let start = Instant::now();

        let (commands_send, commands_recv) = mpsc::channel();
        let (events_send, _) = mpsc::channel();

        thread::spawn(move || nes.run(commands_recv, events_send));
        commands_send.send(Command::Resume)?;

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
                        commands_send.send(Command::Press(keycode_binding(key)))?;
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        commands_send.send(Command::Release(keycode_binding(key)))?;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Runtime for Sdl {
    fn run(log_level: log::Level) -> Result<()> {
        env_logger::builder()
            .target(env_logger::Target::Stdout)
            .filter_level(log_level.to_level_filter())
            .init();

        let (audio_sink, audio_source) = audio_pipeline();
        let sdl_context = sdl2::init().anyhow()?;
        let _speaker = SDLSpeaker::new(&sdl_context, audio_source)?;
        Self::run_with(&sdl_context, audio_sink)
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

pub struct SDLDisplay {
    canvas: WindowCanvas,
    texture: Texture,
    buffer: FrontBuffer,
}

impl SDLDisplay {
    pub fn new(sdl_context: &sdl2::Sdl, buffer: FrontBuffer) -> Result<Self> {
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

pub struct SDLSpeaker<CB: AudioCallback> {
    _device: AudioDevice<CB>,
}

impl<CB: AudioCallback> SDLSpeaker<CB> {
    pub fn new(sdl_context: &sdl2::Sdl, callback: CB) -> Result<Self> {
        let audio = sdl_context.audio().anyhow()?;

        let spec = AudioSpecDesired {
            freq: Some(TARGET_AUDIO_FREQ as i32),
            channels: Some(1),
            samples: Some(AUDIO_SAMPLE_SIZE as u16),
        };

        let device = audio.open_playback(None, &spec, |_| callback).anyhow()?;
        device.resume();

        Ok(Self { _device: device })
    }
}

impl AudioCallback for AudioSource {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.read(out);
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
