#![allow(dead_code)] // Might be disabled by features
use crate::{runtime::Runtime, BufferDisplay, Buttons, INes, HEIGHT, NES, WIDTH};
use anyhow::{anyhow, Context};
use std::{cell::RefCell, error::Error, rc::Rc};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*, Clamped};
use web_sys::{
    js_sys::{ArrayBuffer, Uint8Array},
    CanvasRenderingContext2d, DragEvent, HtmlCanvasElement, ImageData, KeyboardEvent, Window,
};

const ROM: &[u8] = include_bytes!("../../roms/AlwasAwakening_demo.nes");
const MS_PER_FRAME: f64 = 1000.0 / 60.0;

pub struct Web;

impl Runtime for Web {
    fn run() -> Result<(), Box<dyn Error>> {
        console_log::init_with_level(log::Level::Debug)
            .map_err(|_| anyhow!("Failed to initialize logger"))?;

        let ines = INes::read(ROM)?;
        let cartridge = ines.into_cartridge();
        let display = BufferDisplay::default();
        let nes = Rc::new(RefCell::new(NES::new(cartridge, display)));

        let nesdown = nes.clone();
        add_event_listener("keydown", move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            nesdown.borrow_mut().controller().press(button);
            Ok(())
        })?;

        let nesup = nes.clone();
        add_event_listener("keyup", move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            nesup.borrow_mut().controller().release(button);
            Ok(())
        })?;

        let nesdrop = nes.clone();
        add_event_listener("drop", move |event: DragEvent| {
            event.prevent_default();
            let items = event.data_transfer().context("No data transfered")?.items();

            for i in 0..items.length() {
                let item = items.get(i).context("No data transfer item found")?;
                if let Some(file) = item
                    .get_as_file()
                    .map_err(|_| anyhow!("Failed to get file"))?
                {
                    let nesread = nesdrop.clone();

                    let success = closure(move |array_buffer: JsValue| {
                        let array_buffer = array_buffer
                            .dyn_into::<ArrayBuffer>()
                            .map_err(|_| anyhow!("Failed to convert to ArrayBuffer"))?;

                        let array = Uint8Array::new(&array_buffer);
                        let mut rom = vec![0; array.length() as usize];
                        array.copy_to(&mut rom);

                        let ines = INes::read(&mut rom.as_slice())?;
                        let cartridge = ines.into_cartridge();
                        let display = BufferDisplay::default();
                        let nes_new = NES::new(cartridge, display);

                        nesread.replace(nes_new);
                        Ok(())
                    });
                    let failure = closure(move |_| {
                        log::error!("An error occurred getting array buffer");
                        Ok(())
                    });

                    let _ = file.array_buffer().then2(&success, &failure);
                    success.forget();
                    failure.forget();
                }
            }

            Ok(())
        })?;

        add_event_listener("dragover", move |event: DragEvent| {
            event.prevent_default();
            Ok(())
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

fn add_event_listener<T: FromWasmAbi + 'static>(
    event: &str,
    listener: impl FnMut(T) -> Result<(), Box<dyn Error>> + 'static,
) -> anyhow::Result<()> {
    let closure = closure(listener);
    window()?
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .map_err(|_| anyhow!("failed to add event listener"))?;
    // Make closure live forever
    closure.forget();
    Ok(())
}

fn closure<T: FromWasmAbi + 'static>(
    mut function: impl FnMut(T) -> Result<(), Box<dyn Error>> + 'static,
) -> Closure<dyn FnMut(T)> {
    Closure::<dyn FnMut(T)>::new(move |arg| {
        if let Err(err) = function(arg) {
            log::error!("Error: {}", err);
        }
    })
}
