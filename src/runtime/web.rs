#![allow(dead_code)] // Might be disabled by features
use crate::{runtime::Runtime, BufferDisplay, Buttons, INes, HEIGHT, NES, WIDTH};
use anyhow::{anyhow, Context};
use std::{cell::RefCell, error::Error, rc::Rc};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, KeyboardEvent, Window};

const ROM: &[u8] = include_bytes!("../../roms/AlwasAwakening_demo.nes");
const MS_PER_FRAME: f64 = 1000.0 / 60.0;

pub struct Web;

impl Runtime for Web {
    fn run() -> Result<(), Box<dyn Error>> {
        console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");

        let ines = INes::read(ROM)?;
        let cartridge = ines.into_cartridge();
        let display = BufferDisplay::default();
        let nes = Rc::new(RefCell::new(NES::new(cartridge, display)));

        let nesdown = nes.clone();
        add_event_listener("keydown", move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            nesdown.borrow_mut().controller().press(button);
        })?;

        let nesup = nes.clone();
        add_event_listener("keyup", move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            nesup.borrow_mut().controller().release(button);
        })?;

        let context = canvas_context()?;

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let mut timestamp_last_frame_ms = 0.0;

        *g.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
            request_animation_frame(f.borrow().as_ref().unwrap())
                .expect("failed to request animation frame");

            if timestamp_ms - timestamp_last_frame_ms < MS_PER_FRAME {
                return;
            }
            timestamp_last_frame_ms = timestamp_ms;

            // Run NES until frame starts
            let mut nes = nes.borrow_mut();
            while nes.display().vblank() {
                nes.tick();
            }
            // Run NES until frame ends
            while !nes.display().vblank() {
                nes.tick();
            }

            let image_data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(nes.display().buffer()),
                WIDTH as u32,
                HEIGHT as u32,
            )
            .expect("failed to create image data");
            context
                .put_image_data(&image_data, 0.0, 0.0)
                .expect("failed to put image data");
        }));

        request_animation_frame(g.borrow().as_ref().unwrap())?;

        Ok(())
    }
}

fn keycode_binding(keycode: &str) -> Buttons {
    match keycode {
        "KeyZ" => Buttons::A,
        "KeyX" => Buttons::B,
        "ShiftRight" => Buttons::SELECT,
        "Enter" => Buttons::START,
        "ArrowUp" => Buttons::UP,
        "ArrowDown" => Buttons::DOWN,
        "ArrowLeft" => Buttons::LEFT,
        "ArrowRight" => Buttons::RIGHT,
        _ => Buttons::empty(),
    }
}

fn window() -> anyhow::Result<Window> {
    web_sys::window().context("no global `window` exists")
}

fn canvas_context() -> anyhow::Result<CanvasRenderingContext2d> {
    let dom = window()?.document().context("DOM not found")?;
    let canvas = dom
        .get_element_by_id("canvas")
        .context("canvas not found")?;
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| anyhow!("canvas was not a HtmlCanvasElement"))?;

    canvas
        .get_context("2d")
        .map_err(|_| anyhow!("Failed to get canvas context"))?
        .context("Unsupported canvas context '2d'")?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| anyhow!("canvas context was not a CanvasRenderingContext2d"))
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> anyhow::Result<i32> {
    window()?
        .request_animation_frame(f.as_ref().unchecked_ref())
        .map_err(|_| anyhow!("failed to request animation frame"))
}

fn add_event_listener(
    event: &str,
    listener: impl FnMut(KeyboardEvent) + 'static,
) -> anyhow::Result<()> {
    let closure = Closure::<dyn FnMut(KeyboardEvent)>::new(listener);
    window()?
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .map_err(|_| anyhow!("failed to add event listener"))?;
    // Make closure live forever
    closure.forget();
    Ok(())
}
