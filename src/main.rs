use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::time::Instant;

use env_logger::fmt::Target;
use log::{info, LevelFilter};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;

use nes_rust::Button;
use nes_rust::INes;
use nes_rust::NES;
use nes_rust::NESDisplay;

type SDLColor = sdl2::pixels::Color;
type PPUColor = nes_rust::Color;

const WIDTH: u16 = 256;
const HEIGHT: u16 = 240;
const SCALE: u16 = 3;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().target(Target::Stdout).filter_level(LevelFilter::Info).init();

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

    canvas.set_draw_color(SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    canvas.set_scale(f32::from(SCALE), f32::from(SCALE))?;
    canvas.clear();
    canvas.present();

    let display = SDLDisplay::new(canvas);

    let args: Vec<String> = std::env::args().collect();

    let ines = if let Some(filename) = args.get(1) {
        let file = File::open(filename)?;
        INes::read(file)?
    } else {
        let stdin = std::io::stdin();
        let handle = stdin.lock();
        INes::read(handle)?
    };

    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge, display);

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
                    if let Some(button) = keycode_binding(keycode) {
                        nes.controller().press(button);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(button) = keycode_binding(keycode) {
                        nes.controller().release(button);
                    }
                }
                _ => {}
            }
        }
    }
}

fn keycode_binding(keycode: Keycode) -> Option<Button> {
    let button = match keycode {
        Keycode::Z => Button::A,
        Keycode::X => Button::B,
        Keycode::RShift => Button::Select,
        Keycode::Return => Button::Start,
        Keycode::Up => Button::Up,
        Keycode::Down => Button::Down,
        Keycode::Left => Button::Left,
        Keycode::Right => Button::Right,
        _ => return None,
    };

    Some(button)
}

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS);

struct SDLDisplay {
    canvas: Canvas<Window>,
    x: i32,
    y: i32,
    start_of_frame: Instant,
}

impl SDLDisplay {
    fn new(canvas: WindowCanvas) -> Self {
        let now = Instant::now();
        // We start at the LAST tile, because the PPU is always loading data one tile ahead
        SDLDisplay {
            canvas,
            x: i32::from(WIDTH) - 8,
            y: i32::from(HEIGHT),
            start_of_frame: now,
            last_fps_log: now,
            frames_since_last_fps_log: 0,
        }
    }
}

impl NESDisplay for SDLDisplay {
    fn draw_pixel(&mut self, color: PPUColor) {
        self.canvas.set_draw_color(ppu_to_sdl(color));
        self.canvas.draw_point(Point::new(self.x, self.y)).unwrap();
        self.x += 1;

        if self.x == i32::from(WIDTH) {
            self.x = 0;
            self.y += 1;
        }
        if self.y == i32::from(HEIGHT) {
            self.y = 0;
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
}

fn ppu_to_sdl(color: PPUColor) -> SDLColor {
    COLOR_LOOKUP[color.to_byte() as usize]
}

const COLOR_LOOKUP: [SDLColor; 64] = [
    SDLColor {
        r: 84,
        g: 84,
        b: 84,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 30,
        b: 116,
        a: 255,
    },
    SDLColor {
        r: 8,
        g: 16,
        b: 144,
        a: 255,
    },
    SDLColor {
        r: 48,
        g: 0,
        b: 136,
        a: 255,
    },
    SDLColor {
        r: 68,
        g: 0,
        b: 100,
        a: 255,
    },
    SDLColor {
        r: 92,
        g: 0,
        b: 48,
        a: 255,
    },
    SDLColor {
        r: 84,
        g: 4,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 60,
        g: 24,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 32,
        g: 42,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 8,
        g: 58,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 64,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 60,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 50,
        b: 60,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 152,
        g: 150,
        b: 152,
        a: 255,
    },
    SDLColor {
        r: 8,
        g: 76,
        b: 196,
        a: 255,
    },
    SDLColor {
        r: 48,
        g: 50,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 92,
        g: 30,
        b: 228,
        a: 255,
    },
    SDLColor {
        r: 136,
        g: 20,
        b: 176,
        a: 255,
    },
    SDLColor {
        r: 160,
        g: 20,
        b: 100,
        a: 255,
    },
    SDLColor {
        r: 152,
        g: 34,
        b: 32,
        a: 255,
    },
    SDLColor {
        r: 120,
        g: 60,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 84,
        g: 90,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 40,
        g: 114,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 8,
        g: 124,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 118,
        b: 40,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 102,
        b: 120,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 238,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 76,
        g: 154,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 120,
        g: 124,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 176,
        g: 98,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 228,
        g: 84,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 88,
        b: 180,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 106,
        b: 100,
        a: 255,
    },
    SDLColor {
        r: 212,
        g: 136,
        b: 32,
        a: 255,
    },
    SDLColor {
        r: 160,
        g: 170,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 116,
        g: 196,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 76,
        g: 208,
        b: 32,
        a: 255,
    },
    SDLColor {
        r: 56,
        g: 204,
        b: 108,
        a: 255,
    },
    SDLColor {
        r: 56,
        g: 180,
        b: 204,
        a: 255,
    },
    SDLColor {
        r: 60,
        g: 60,
        b: 60,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 238,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 168,
        g: 204,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 188,
        g: 188,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 212,
        g: 178,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 174,
        b: 236,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 174,
        b: 212,
        a: 255,
    },
    SDLColor {
        r: 236,
        g: 180,
        b: 176,
        a: 255,
    },
    SDLColor {
        r: 228,
        g: 196,
        b: 144,
        a: 255,
    },
    SDLColor {
        r: 204,
        g: 210,
        b: 120,
        a: 255,
    },
    SDLColor {
        r: 180,
        g: 222,
        b: 120,
        a: 255,
    },
    SDLColor {
        r: 168,
        g: 226,
        b: 144,
        a: 255,
    },
    SDLColor {
        r: 152,
        g: 226,
        b: 180,
        a: 255,
    },
    SDLColor {
        r: 160,
        g: 214,
        b: 228,
        a: 255,
    },
    SDLColor {
        r: 160,
        g: 162,
        b: 160,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
    SDLColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    },
];
