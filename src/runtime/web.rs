#![allow(dead_code)] // Might be disabled by features
use super::FRAME_DURATION;
use crate::audio::TARGET_AUDIO_FREQ;
use crate::{runtime::Runtime, BufferDisplay, Buttons, INes, NESSpeaker, HEIGHT, NES, WIDTH};
use anyhow::Result;
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
    js_sys,
    js_sys::{ArrayBuffer, Uint8Array},
    CanvasRenderingContext2d, Document, DragEvent, Event, EventTarget, File, HtmlCanvasElement,
    HtmlInputElement, ImageData, KeyboardEvent, PointerEvent, Storage, Window,
};
use zip::ZipArchive;

const DEFAULT_ROM: &[u8] = include_bytes!("../../roms/AlwasAwakening_demo.nes");

// True frequency is 1789773Hz, but tuned to match my emulator's rate
// TODO: may have to change this to use NES_FREQ
const NES_AUDIO_FREQ: f64 = 1_866_000.0;

pub struct Web;

impl Runtime for Web {
    fn init_log(level: log::Level) -> Result<()> {
        std::panic::set_hook(Box::new(|info| {
            web_sys::console::error_1(&info.to_string().into());
        }));
        console_log::init_with_level(level)?;
        Ok(())
    }

    fn run() -> Result<()> {
        let window = window()?;
        let dom = dom()?;

        let rom = load_rom()?;
        let ctx = set_rom(&rom)?;
        let ctx = Rc::new(RefCell::new(ctx));

        add_event_listener(&window, "keydown", {
            let ctx = ctx.clone();
            move |event: KeyboardEvent| {
                let nes = &mut ctx.borrow_mut().nes;
                let button = keycode_binding(&event.code());
                nes.controller().press(button);
                Ok(())
            }
        })?;

        add_event_listener(&window, "keyup", {
            let ctx = ctx.clone();
            move |event: KeyboardEvent| {
                let nes = &mut ctx.borrow_mut().nes;
                let button = keycode_binding(&event.code());
                nes.controller().release(button);
                Ok(())
            }
        })?;

        let controller_element = dom
            .get_element_by_id("controller")
            .context("controller not found")?;

        add_event_listener(&controller_element, "contextmenu", |event: PointerEvent| {
            event.prevent_default();
            Ok(())
        })?;

        const BUTTONS: [(&str, Buttons); 8] = [
            ("a", Buttons::A),
            ("b", Buttons::B),
            ("start", Buttons::START),
            ("select", Buttons::SELECT),
            ("up", Buttons::UP),
            ("down", Buttons::DOWN),
            ("left", Buttons::LEFT),
            ("right", Buttons::RIGHT),
        ];

        for (button_id, button) in BUTTONS.iter() {
            let element = dom
                .get_element_by_id(button_id)
                .context(format!("button not found {button_id}"))?;

            add_event_listener(&element, "pointerenter", {
                let ctx = ctx.clone();
                move |_: PointerEvent| {
                    let nes = &mut ctx.borrow_mut().nes;
                    nes.controller().press(*button);
                    Ok(())
                }
            })?;

            add_event_listener(&element, "pointerout", {
                let ctx = ctx.clone();
                move |_: PointerEvent| {
                    let nes = &mut ctx.borrow_mut().nes;
                    nes.controller().release(*button);
                    Ok(())
                }
            })?;
        }

        let upload_button = dom
            .get_element_by_id("upload-rom")
            .context("upload-rom button not found")?
            .dyn_into::<HtmlInputElement>()
            .map_err(|_| anyhow!("upload-rom button was not a HtmlInputElement"))?;

        add_event_listener(&upload_button.clone(), "change", {
            let ctx = ctx.clone();
            move |_: Event| {
                let file_list = upload_button.files().context("No files selected")?;
                let mut files = vec![];
                for i in 0..file_list.length() {
                    let file = file_list.item(i).context("No file found")?;
                    files.push(file);
                }
                upload_rom(ctx.clone(), files.into_iter());
                Ok(())
            }
        })?;

        add_event_listener(&window, "drop", {
            let ctx = ctx.clone();
            move |event: DragEvent| {
                event.prevent_default();
                let items = event
                    .data_transfer()
                    .context("No data transferred")?
                    .items();

                let mut files = vec![];
                for i in 0..items.length() {
                    let item = items.get(i).context("No data transfer item found")?;
                    if let Some(file) = item.get_as_file().anyhow()? {
                        files.push(file);
                    }
                }

                upload_rom(ctx.clone(), files.into_iter());
                Ok(())
            }
        })?;

        add_event_listener(&window, "dragover", move |event: DragEvent| {
            event.prevent_default();
            Ok(())
        })?;

        let context = canvas_context()?;

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let mut timestamp_start_ms = 0.0;
        let mut num_frames: u64 = 0;

        *g.borrow_mut() = Some(closure({
            let ctx = ctx.clone();
            move |timestamp_ms: f64| {
                request_animation_frame(f.borrow().as_ref().unwrap())?;

                if timestamp_start_ms == 0.0 {
                    timestamp_start_ms = timestamp_ms;
                }

                let expected_frames = ((timestamp_ms - timestamp_start_ms)
                    / FRAME_DURATION.as_millis() as f64)
                    as u64;

                let needed_frames = (expected_frames - num_frames).min(3);
                if needed_frames == 0 {
                    return Ok(());
                }

                let mut ctx = ctx.borrow_mut();
                let NesContext { nes, rom_hash } = &mut *ctx;

                // Save state every frame, inefficient but it doesn't seem to matter
                save_state(*rom_hash, nes)?;

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
                .anyhow()?;
                context.put_image_data(&image_data, 0.0, 0.0).anyhow()?;
                Ok(())
            }
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
        "KeyZ" | "KeyA" => Buttons::A,
        "KeyX" | "KeyS" => Buttons::B,
        "ShiftRight" | "ShiftLeft" => Buttons::SELECT,
        "Enter" => Buttons::START,
        "ArrowUp" => Buttons::UP,
        "ArrowDown" => Buttons::DOWN,
        "ArrowLeft" => Buttons::LEFT,
        "ArrowRight" => Buttons::RIGHT,
        _ => Buttons::empty(),
    }
}

fn window() -> Result<Window> {
    web_sys::window().context("no global `window` exists")
}

fn dom() -> Result<Document> {
    window()?.document().context("DOM not found")
}

fn canvas_context() -> Result<CanvasRenderingContext2d> {
    let canvas = dom()?
        .get_element_by_id("canvas")
        .context("canvas not found")?;
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| anyhow!("canvas was not a HtmlCanvasElement"))?;

    Ok(canvas
        .get_context("2d")
        .anyhow()?
        .context("Unsupported canvas context '2d'")?
        .unchecked_into::<CanvasRenderingContext2d>())
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> Result<i32> {
    let result = window()?
        .request_animation_frame(f.as_ref().unchecked_ref())
        .anyhow()?;
    Ok(result)
}

fn add_event_listener<T: FromWasmAbi + 'static>(
    target: &EventTarget,
    event: &str,
    listener: impl FnMut(T) -> Result<()> + 'static,
) -> Result<()> {
    let closure = closure(listener);
    target
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .anyhow()?;
    // Make closure live forever
    closure.forget();
    Ok(())
}

fn closure<T: FromWasmAbi + 'static>(
    mut function: impl FnMut(T) -> Result<()> + 'static,
) -> Closure<dyn FnMut(T)> {
    Closure::<dyn FnMut(T)>::new(move |arg| {
        if let Err(err) = function(arg) {
            log::error!("Error: {}", err);
        }
    })
}

fn upload_rom(ctx: Rc<RefCell<NesContext>>, files: impl Iterator<Item = File>) {
    for file in files {
        let filename = file.name();
        let ctx = ctx.clone();

        let success = closure(move |array_buffer: JsValue| {
            let array_buffer = array_buffer.unchecked_into::<ArrayBuffer>();

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
            ctx.replace(new_ctx);
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

fn set_rom(rom: &[u8]) -> Result<NesContext> {
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

fn save_state<D, S>(rom_hash: u64, nes: &mut NES<D, S>) -> Result<()> {
    let ram = nes.cpu.memory().prg().ram();

    let key = state_key(rom_hash);
    let value = BASE64_STANDARD.encode(ram);
    local_storage()?.set_item(&key, &value).anyhow()?;
    Ok(())
}

fn load_state<D, S>(rom_hash: u64, nes: &mut NES<D, S>) -> Result<()> {
    let key = state_key(rom_hash);
    let value = match local_storage()?.get_item(&key).anyhow()? {
        Some(value) => value,
        None => return Ok(()), // No state saved
    };

    let ram = BASE64_STANDARD.decode(value)?;
    nes.cpu.memory().prg().ram().copy_from_slice(&ram);
    Ok(())
}

const ROM_KEY: &str = "nes-rom";

fn save_rom(rom: &[u8]) -> Result<()> {
    let value = BASE64_STANDARD.encode(rom);
    local_storage()?.set_item(ROM_KEY, &value).anyhow()?;
    Ok(())
}

fn load_rom() -> Result<Vec<u8>> {
    let value = match local_storage()?.get_item(ROM_KEY).anyhow()? {
        Some(value) => value,
        None => return Ok(DEFAULT_ROM.to_vec()),
    };
    Ok(BASE64_STANDARD.decode(value)?)
}

fn local_storage() -> Result<Storage> {
    Ok(window()?
        .local_storage()
        .anyhow()?
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
    fn emit(&mut self, value: f32) {
        // Naive downsampling
        if self.next_sample <= 0.0 {
            push_audio_buffer(value);
            self.next_sample += NES_AUDIO_FREQ / TARGET_AUDIO_FREQ;
        }
        self.next_sample -= 1.0;
    }
}

#[wasm_bindgen(module = "/web/audio.js")]
extern "C" {
    #[wasm_bindgen(js_name = pushAudioBuffer)]
    fn push_audio_buffer(byte: f32);
}

trait WebResult<T> {
    fn anyhow(self) -> Result<T>;
}

impl<T> WebResult<T> for Result<T, JsValue> {
    fn anyhow(self) -> Result<T> {
        self.map_err(|e| {
            let dbg = format!("{:?}", e);
            let msg = js_sys::Error::from(e).message().as_string().unwrap_or(dbg);
            anyhow!(msg)
        })
    }
}
