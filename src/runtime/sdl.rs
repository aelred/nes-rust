use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::time::Instant;

use log::info;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use crate::INes;
use crate::NESDisplay;
use crate::NES;
use crate::{Buttons, Color, HEIGHT, WIDTH};

use super::Runtime;
use super::FRAME_DURATION;

const SCALE: u16 = 3;

pub struct Sdl;

impl Runtime for Sdl {
    fn init_log(level: log::Level) -> Result<(), Box<dyn Error>> {
        env_logger::builder()
            .target(env_logger::Target::Stdout)
            .filter_level(level.to_level_filter())
            .init();
        Ok(())
    }

    fn run() -> Result<(), Box<dyn Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let mut event_pump = sdl_context.event_pump()?;

        let window = video_subsystem
            .window(
                "nes-rust",
                u32::from(WIDTH * SCALE),
                u32::from(HEIGHT * SCALE),
            )
            .position_centered()
            .build()?;

        let mut canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()?;

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
        let display = SDLDisplay::new(&texture_creator, canvas);

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

        let mut nes = NES::new(cartridge, display);

        loop {
            // Arbitrary number of ticks so we don't poll events too much
            for _ in 1..1000 {
                nes.tick();
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

fn keycode_binding(keycode: Keycode) -> Buttons {
    match keycode {
        Keycode::Z => Buttons::A,
        Keycode::X => Buttons::B,
        Keycode::RShift => Buttons::SELECT,
        Keycode::Return => Buttons::START,
        Keycode::Up => Buttons::UP,
        Keycode::Down => Buttons::DOWN,
        Keycode::Left => Buttons::LEFT,
        Keycode::Right => Buttons::RIGHT,
        _ => Buttons::empty(),
    }
}

struct SDLDisplay<'r> {
    canvas: WindowCanvas,
    texture: Texture<'r>,
    buffer: [u8; WIDTH as usize * HEIGHT as usize * 4],
    x: usize,
    y: usize,
    start_of_frame: Instant,
    last_fps_log: Instant,
    frames_since_last_fps_log: u64,
}

impl<'r> SDLDisplay<'r> {
    fn new(texture_creator: &'r TextureCreator<WindowContext>, canvas: WindowCanvas) -> Self {
        let texture = texture_creator
            .create_texture_streaming(None, WIDTH as u32, HEIGHT as u32)
            .unwrap();

        let now = Instant::now();
        // We start at the LAST tile, because the PPU is always loading data one tile ahead
        SDLDisplay {
            canvas,
            texture,
            buffer: [0; WIDTH as usize * HEIGHT as usize * 4],
            x: usize::from(WIDTH) - 8,
            y: usize::from(HEIGHT) - 1,
            start_of_frame: now,
            last_fps_log: now,
            frames_since_last_fps_log: 0,
        }
    }
}

impl<'r> NESDisplay for SDLDisplay<'r> {
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

            let elapsed = self.start_of_frame.elapsed();
            let time_to_sleep = FRAME_DURATION.checked_sub(elapsed).unwrap_or_default();
            std::thread::sleep(time_to_sleep);

            self.start_of_frame = Instant::now();

            self.frames_since_last_fps_log += 1;

            let elapsed_since_last_fps_log = self.last_fps_log.elapsed();
            if elapsed_since_last_fps_log > Duration::from_secs(5) {
                let fps = self.frames_since_last_fps_log / elapsed_since_last_fps_log.as_secs();
                info!("FPS: {}", fps);
                self.last_fps_log = Instant::now();
                self.frames_since_last_fps_log = 0;
            }
        }
    }

    fn enter_vblank(&mut self) {}
}
