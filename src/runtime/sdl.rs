use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use super::Runtime;
use crate::audio::{audio_pipeline, AudioSource, AUDIO_SAMPLE_SIZE, TARGET_AUDIO_FREQ};
use crate::NESDisplay;
use crate::NESSpeaker;
use crate::NES;
use crate::{Buttons, Color, HEIGHT, WIDTH};
use crate::{INes, NES_FREQ};
use log::info;
use sdl2::audio::AudioCallback;
use sdl2::audio::AudioDevice;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::render::WindowCanvas;

const SCALE: u16 = 3;

pub struct Sdl;

impl Sdl {
    pub fn run_with(
        sdl_context: &sdl2::Sdl,
        display: impl NESDisplay,
        speaker: impl NESSpeaker,
    ) -> Result<(), Box<dyn Error>> {
        let mut event_pump = sdl_context.event_pump()?;

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

        let mut nes = NES::new(cartridge, display, speaker);
        let mut cycles: u64 = 0;
        let start = Instant::now();

        loop {
            // Arbitrary number of ticks so we don't poll events or sleep too regularly
            for _ in 1..1000 {
                cycles += nes.tick() as u64;
            }

            let expected_time = Duration::from_secs_f64(cycles as f64 / NES_FREQ);
            let actual_time = start.elapsed();
            if actual_time < expected_time {
                thread::sleep(expected_time - actual_time);
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        return Ok(());
                    }
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        nes.controller().press(keycode_binding(keycode));
                    }
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        nes.controller().release(keycode_binding(keycode));
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Runtime for Sdl {
    fn init_log(level: log::Level) -> Result<(), Box<dyn Error>> {
        env_logger::builder()
            .target(env_logger::Target::Stdout)
            .filter_level(level.to_level_filter())
            .init();
        Ok(())
    }

    fn run() -> Result<(), Box<dyn Error>> {
        let (audio_sink, audio_source) = audio_pipeline();
        let sdl_context = sdl2::init()?;
        let display = SDLDisplay::new(&sdl_context)?;
        let _speaker = SDLSpeaker::new(&sdl_context, audio_source)?;
        Self::run_with(&sdl_context, display, audio_sink)
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
    buffer: [u8; WIDTH as usize * HEIGHT as usize * 4],
    x: usize,
    y: usize,
    last_fps_log: Instant,
    frames_since_last_fps_log: u64,
}

impl SDLDisplay {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "nes-rust",
                u32::from(WIDTH * SCALE),
                u32::from(HEIGHT * SCALE),
            )
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().target_texture().build()?;

        canvas.set_draw_color(sdl2::pixels::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        canvas.set_scale(f32::from(SCALE), f32::from(SCALE))?;
        canvas.clear();
        canvas.present();

        let texture_creator = canvas.texture_creator();
        let texture =
            texture_creator.create_texture_streaming(None, WIDTH as u32, HEIGHT as u32)?;

        let now = Instant::now();
        Ok(Self {
            canvas,
            texture,
            buffer: [0; WIDTH as usize * HEIGHT as usize * 4],
            x: 0,
            y: 0,
            last_fps_log: now,
            frames_since_last_fps_log: 0,
        })
    }
}

impl NESDisplay for SDLDisplay {
    fn draw_pixel(&mut self, color: Color) {
        let offset = (self.y * WIDTH as usize + self.x) * 4;
        if offset + 2 < self.buffer.len() {
            let (r, g, b) = color.to_rgb();
            self.buffer[offset] = b;
            self.buffer[offset + 1] = g;
            self.buffer[offset + 2] = r;
        }

        self.x += 1;

        if self.x == usize::from(WIDTH) {
            self.x = 0;
            self.y += 1;
        }
        if self.y == usize::from(HEIGHT) {
            self.y = 0;
            self.texture
                .update(None, &self.buffer, WIDTH as usize * 4)
                .unwrap();
            self.canvas.copy(&self.texture, None, None).unwrap();
            self.canvas.present();

            self.frames_since_last_fps_log += 1;

            let now = Instant::now();
            let elapsed_since_last_fps_log = now.duration_since(self.last_fps_log);
            if elapsed_since_last_fps_log > Duration::from_secs(5) {
                let fps = self.frames_since_last_fps_log as f64
                    / elapsed_since_last_fps_log.as_secs_f64();
                info!("FPS: {}", fps);
                self.last_fps_log = now;
                self.frames_since_last_fps_log = 0;
            }
        }
    }

    fn enter_vblank(&mut self) {}
}

pub struct SDLSpeaker {
    _device: AudioDevice<AudioSource>,
}

impl SDLSpeaker {
    pub fn new(sdl_context: &sdl2::Sdl, audio_source: AudioSource) -> Result<Self, String> {
        let audio_subsystem = sdl_context.audio()?;

        let spec = AudioSpecDesired {
            freq: Some(TARGET_AUDIO_FREQ as i32),
            channels: Some(1),
            samples: Some(AUDIO_SAMPLE_SIZE as u16),
        };

        let device = audio_subsystem.open_playback(None, &spec, |_| audio_source)?;
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
