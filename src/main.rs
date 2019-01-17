use std::error::Error;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use nes_rust::INes;
use nes_rust::NES;
use nes_rust::NESDisplay;

type SDLColor = sdl2::pixels::Color;
type PPUColor = nes_rust::Color;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let window = video_subsystem
        .window("nes-rust", 256, 240)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()?;

    canvas.set_draw_color(SDLColor { r: 0, g: 0, b: 0, a: 255 });
    canvas.clear();
    canvas.present();

    let display = SDLDisplay {
        canvas,
        x: 0,
        y: 0,
    };

    let stdin = std::io::stdin();
    let handle = stdin.lock();

    let ines = INes::read(handle)?;
    let mut cartridge = ines.into_cartridge();

    let mut nes = NES::new(&mut cartridge, display);

    loop {
        nes.tick();
        std::thread::sleep(Duration::from_micros(1));

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

struct SDLDisplay {
    canvas: Canvas<Window>,
    x: i32,
    y: i32,
}

impl NESDisplay for SDLDisplay {
    fn draw_pixel(&mut self, color: PPUColor) {
        self.canvas.set_draw_color(ppu_to_sdl(color));
        self.canvas.draw_point(Point::new(self.x, self.y)).unwrap();
        self.x += 1;

        if self.x == 256 {
            self.x = 0;
            self.y += 1;
        }
        if self.y == 240 {
            self.y = 0;
            self.canvas.present();
        }
    }
}

fn ppu_to_sdl(color: PPUColor) -> SDLColor {
    COLOR_LOOKUP[color.to_byte() as usize]
}

const COLOR_LOOKUP: [SDLColor; 56] = [
    SDLColor { r: 84, g: 84, b: 84, a: 255 }, SDLColor { r: 0, g: 30, b: 116, a: 255 }, SDLColor { r: 8, g: 16, b: 144, a: 255 }, SDLColor { r: 48, g: 0, b: 136, a: 255 }, SDLColor { r: 68, g: 0, b: 100, a: 255 }, SDLColor { r: 92, g: 0, b: 48, a: 255 }, SDLColor { r: 84, g: 4, b: 0, a: 255 }, SDLColor { r: 60, g: 24, b: 0, a: 255 }, SDLColor { r: 32, g: 42, b: 0, a: 255 }, SDLColor { r: 8, g: 58, b: 0, a: 255 }, SDLColor { r: 0, g: 64, b: 0, a: 255 }, SDLColor { r: 0, g: 60, b: 0, a: 255 }, SDLColor { r: 0, g: 50, b: 60, a: 255 }, SDLColor { r: 0, g: 0, b: 0, a: 255 },
    SDLColor { r: 152, g: 150, b: 152, a: 255 }, SDLColor { r: 8, g: 76, b: 196, a: 255 }, SDLColor { r: 48, g: 50, b: 236, a: 255 }, SDLColor { r: 92, g: 30, b: 228, a: 255 }, SDLColor { r: 136, g: 20, b: 176, a: 255 }, SDLColor { r: 160, g: 20, b: 100, a: 255 }, SDLColor { r: 152, g: 34, b: 32, a: 255 }, SDLColor { r: 120, g: 60, b: 0, a: 255 }, SDLColor { r: 84, g: 90, b: 0, a: 255 }, SDLColor { r: 40, g: 114, b: 0, a: 255 }, SDLColor { r: 8, g: 124, b: 0, a: 255 }, SDLColor { r: 0, g: 118, b: 40, a: 255 }, SDLColor { r: 0, g: 102, b: 120, a: 255 }, SDLColor { r: 0, g: 0, b: 0, a: 255 },
    SDLColor { r: 236, g: 238, b: 236, a: 255 }, SDLColor { r: 76, g: 154, b: 236, a: 255 }, SDLColor { r: 120, g: 124, b: 236, a: 255 }, SDLColor { r: 176, g: 98, b: 236, a: 255 }, SDLColor { r: 228, g: 84, b: 236, a: 255 }, SDLColor { r: 236, g: 88, b: 180, a: 255 }, SDLColor { r: 236, g: 106, b: 100, a: 255 }, SDLColor { r: 212, g: 136, b: 32, a: 255 }, SDLColor { r: 160, g: 170, b: 0, a: 255 }, SDLColor { r: 116, g: 196, b: 0, a: 255 }, SDLColor { r: 76, g: 208, b: 32, a: 255 }, SDLColor { r: 56, g: 204, b: 108, a: 255 }, SDLColor { r: 56, g: 180, b: 204, a: 255 }, SDLColor { r: 60, g: 60, b: 60, a: 255 },
    SDLColor { r: 236, g: 238, b: 236, a: 255 }, SDLColor { r: 168, g: 204, b: 236, a: 255 }, SDLColor { r: 188, g: 188, b: 236, a: 255 }, SDLColor { r: 212, g: 178, b: 236, a: 255 }, SDLColor { r: 236, g: 174, b: 236, a: 255 }, SDLColor { r: 236, g: 174, b: 212, a: 255 }, SDLColor { r: 236, g: 180, b: 176, a: 255 }, SDLColor { r: 228, g: 196, b: 144, a: 255 }, SDLColor { r: 204, g: 210, b: 120, a: 255 }, SDLColor { r: 180, g: 222, b: 120, a: 255 }, SDLColor { r: 168, g: 226, b: 144, a: 255 }, SDLColor { r: 152, g: 226, b: 180, a: 255 }, SDLColor { r: 160, g: 214, b: 228, a: 255 }, SDLColor { r: 160, g: 162, b: 160, a: 255 },
];