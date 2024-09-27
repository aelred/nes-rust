#![allow(dead_code)] // Might be disabled by features
use crate::{runtime::Runtime, BufferDisplay, Buttons, INes, NESSpeaker, HEIGHT, NES, WIDTH};
use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_STANDARD, Engine};
use std::{
    cell::RefCell,
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Read},
    rc::Rc,
};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*, Clamped};
use web_sys::{
    js_sys::{ArrayBuffer, Uint8Array},
    CanvasRenderingContext2d, DragEvent, HtmlCanvasElement, ImageData, KeyboardEvent, Storage,
    Window,
};
use zip::ZipArchive;

use super::{FRAME_DURATION, NES_AUDIO_FREQ, TARGET_AUDIO_FREQ};

pub struct Web;

impl Runtime for Web {
    fn init_log(level: log::Level) -> Result<(), Box<dyn Error>> {
        console_log::init_with_level(level).map_err(|_| anyhow!("Failed to initialize logger"))?;
        Ok(())
    }

    fn run() -> Result<(), Box<dyn Error>> {
        let base_ctx = Rc::new(RefCell::new(Option::<NesContext>::None));

        if let Some(rom) = load_rom()? {
            let new_ctx = set_rom(&rom)?;
            base_ctx.borrow_mut().replace(new_ctx);
        }

        let ctx = base_ctx.clone();
        add_event_listener("keydown", move |event: KeyboardEvent| {
            let mut ctx = ctx.borrow_mut();
            let nes = match &mut *ctx {
                Some(ctx) => &mut ctx.nes,
                None => return Ok(()),
            };
            let button = keycode_binding(&event.code());
            nes.controller().press(button);
            Ok(())
        })?;

        let ctx = base_ctx.clone();
        add_event_listener("keyup", move |event: KeyboardEvent| {
            let mut ctx = ctx.borrow_mut();
            let nes = match &mut *ctx {
                Some(ctx) => &mut ctx.nes,
                None => return Ok(()),
            };
            let button = keycode_binding(&event.code());
            nes.controller().release(button);
            Ok(())
        })?;

        let ctx = base_ctx.clone();
        add_event_listener("drop", move |event: DragEvent| {
            event.prevent_default();
            let items = event.data_transfer().context("No data transfered")?.items();

            for i in 0..items.length() {
                let item = items.get(i).context("No data transfer item found")?;
                if let Some(file) = item
                    .get_as_file()
                    .map_err(|_| anyhow!("Failed to get file"))?
                {
                    let filename = file.name();
                    let ctx = ctx.clone();

                    let success = closure(move |array_buffer: JsValue| {
                        let array_buffer = array_buffer
                            .dyn_into::<ArrayBuffer>()
                            .map_err(|_| anyhow!("Failed to convert to ArrayBuffer"))?;

                        let array = Uint8Array::new(&array_buffer);
                        let mut data = vec![0; array.length() as usize];
                        array.copy_to(&mut data);

                        let mut rom: Option<Vec<u8>> = None;

                        if filename.ends_with(".zip") {
                            let mut zip = ZipArchive::new(Cursor::new(data))?;

                            for index in 0..zip.len() {
                                let mut file = zip.by_index(index)?;
                                if file.name().ends_with(".nes") {
                                    let mut rom_data = vec![0; file.size() as usize];
                                    file.read_exact(&mut rom_data)?;
                                    rom = Some(rom_data);
                                    break;
                                }
                            }
                        } else {
                            rom = Some(data);
                        }

                        let rom = rom.ok_or_else(|| anyhow!("No .nes file found"))?;

                        let new_ctx = set_rom(&rom)?;
                        ctx.replace(Some(new_ctx));
                        save_rom(&rom)?;
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

        let mut timestamp_start_ms = 0.0;
        let mut num_frames: u64 = 0;

        let ctx = base_ctx.clone();
        *g.borrow_mut() = Some(closure(move |timestamp_ms: f64| {
            request_animation_frame(f.borrow().as_ref().unwrap())?;

            if timestamp_start_ms == 0.0 {
                timestamp_start_ms = timestamp_ms;
            }

            let expected_frames =
                ((timestamp_ms - timestamp_start_ms) / FRAME_DURATION.as_millis() as f64) as u64;

            let needed_frames = (expected_frames - num_frames).min(3);
            if needed_frames == 0 {
                return Ok(());
            }

            let mut ctx = ctx.borrow_mut();
            let ctx = match &mut *ctx {
                Some(ctx) => ctx,
                None => return Ok(()),
            };
            let nes = &mut ctx.nes;

            // Save state every frame, inefficient but it doesn't seem to matter
            save_state(ctx.rom_hash, nes)?;

            for _ in 0..needed_frames {
                // Run NES until frame starts
                while nes.display().vblank() {
                    nes.tick();
                }
                // Run NES until frame ends
                while !nes.display().vblank() {
                    nes.tick();
                }
            }
            num_frames = expected_frames;

            let image_data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(nes.display().buffer()),
                WIDTH as u32,
                HEIGHT as u32,
            )
            .map_err(|_| anyhow!("Failed to create image data"))?;
            context
                .put_image_data(&image_data, 0.0, 0.0)
                .map_err(|_| anyhow!("Failed to put image data"))?;
            Ok(())
        }));

        request_animation_frame(g.borrow().as_ref().unwrap())?;

        Ok(())
    }
}

struct NesContext {
    nes: NES<BufferDisplay, WebSpeaker>,
    rom_hash: u64,
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

fn set_rom(rom: &[u8]) -> Result<NesContext, Box<dyn Error>> {
    let ines = INes::read(rom)?;
    let cartridge = ines.into_cartridge();
    let display = BufferDisplay::default();
    let speaker = WebSpeaker::default();

    let mut rom_hasher = DefaultHasher::new();
    rom.hash(&mut rom_hasher);
    let rom_hash = rom_hasher.finish();

    let mut nes = NES::new(cartridge, display, speaker);
    load_state(rom_hash, &mut nes)?;

    Ok(NesContext { nes, rom_hash })
}

fn save_state<D, S>(rom_hash: u64, nes: &mut NES<D, S>) -> Result<(), Box<dyn Error>> {
    let ram = nes.cpu.memory().prg().ram();

    let key = state_key(rom_hash);
    let value = BASE64_STANDARD.encode(ram);
    local_storage()?
        .set_item(&key, &value)
        .map_err(|_| anyhow!("Failed to save state to local storage"))?;
    Ok(())
}

fn load_state<D, S>(rom_hash: u64, nes: &mut NES<D, S>) -> Result<(), Box<dyn Error>> {
    let key = state_key(rom_hash);
    let value = match local_storage()?
        .get_item(&key)
        .map_err(|_| anyhow!("Failed to get state from local storage"))?
    {
        Some(value) => value,
        None => return Ok(()), // No state saved
    };

    let ram = BASE64_STANDARD.decode(value)?;
    nes.cpu.memory().prg().ram().copy_from_slice(&ram);
    Ok(())
}

const ROM_KEY: &str = "nes-rom";

fn save_rom(rom: &[u8]) -> Result<(), Box<dyn Error>> {
    let value = BASE64_STANDARD.encode(rom);
    local_storage()?
        .set_item(ROM_KEY, &value)
        .map_err(|_| anyhow!("Failed to save ROM"))?;
    Ok(())
}

fn load_rom() -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    let value = match local_storage()?
        .get_item(ROM_KEY)
        .map_err(|_| anyhow!("Failed to read ROM"))?
    {
        Some(value) => value,
        None => return Ok(None),
    };
    Ok(Some(BASE64_STANDARD.decode(value)?))
}

fn local_storage() -> Result<Storage, Box<dyn Error>> {
    Ok(window()?
        .local_storage()
        .map_err(|_| anyhow!("Failed to get local storage"))?
        .context("Failed to get local storage")?)
}

fn state_key(rom_hash: u64) -> String {
    let hash_base64 = BASE64_STANDARD.encode(rom_hash.to_le_bytes());
    format!("nes-state-{}", hash_base64)
}

#[derive(Default)]
struct WebSpeaker {
    next_sample: f64,
}

impl NESSpeaker for WebSpeaker {
    fn emit(&mut self, value: u8) {
        // Naive downsampling
        if self.next_sample <= 0.0 {
            push_audio_buffer(value);
            self.next_sample += NES_AUDIO_FREQ / TARGET_AUDIO_FREQ as f64;
        }
        self.next_sample -= 1.0;
    }
}

#[wasm_bindgen(module = "/web/audio.js")]
extern "C" {
    #[wasm_bindgen(js_name = pushAudioBuffer)]
    fn push_audio_buffer(byte: u8);
}
