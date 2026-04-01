#![allow(dead_code)] // Might be disabled by features

mod audio;

use crate::runner::{Event, NESRunner};
use crate::runtime::web::audio::wasm_audio;
use crate::video::FrontBuffer;
use crate::{runtime::Runtime, Buttons, INes, HEIGHT, WIDTH};
use anyhow::{anyhow, Context};
use anyhow::{bail, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use js_sys::futures::spawn_local;
use js_sys::{Uint8ClampedArray, WebAssembly};
use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Read},
    rc::Rc,
};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use web_sys::{
    js_sys,
    js_sys::{ArrayBuffer, Uint8Array},
    AudioContext, CanvasRenderingContext2d, Document, DragEvent, EventTarget, File,
    HtmlCanvasElement, HtmlInputElement, ImageData, KeyboardEvent, MouseEvent, PointerEvent,
    Storage, VisibilityState, Window,
};
use zip::ZipArchive;

const DEFAULT_ROM: &[u8] = include_bytes!("../../../roms/AlwasAwakening_demo.nes");

pub struct Web;

impl Runtime for Web {
    fn run(log_level: log::Level) -> Result<()> {
        // Ignore error if logger is already configured - can happen with page reload shenanigans
        let _ = console_log::init_with_level(log_level);
        std::panic::set_hook(Box::new(|info| log::error!("{}", info)));

        spawn_local(async {
            if let Err(e) = run().await {
                log::error!("Error: {}", e);
            }
        });
        Ok(())
    }
}

async fn run() -> Result<()> {
    let window = window()?;
    let dom = document()?;

    let mut ctx = NesContext::new().await?;

    let rom = load_rom()?;
    ctx.set_rom(&rom)?;
    let ctx = Rc::new(RefCell::new(ctx));

    add_event_listener(&window, "visibilitychange", {
        let ctx = ctx.clone();
        move |_: web_sys::Event| ctx.borrow_mut().set_paused_from_visibility()
    })?;

    // Audio is only allowed to start after user interacts with page
    add_event_listener(&window, "keydown", {
        let ctx = ctx.clone();
        move |_: KeyboardEvent| {
            ctx.borrow_mut().start_audio();
            Ok(())
        }
    })?;

    add_event_listener(&window, "click", {
        let ctx = ctx.clone();
        move |_: MouseEvent| {
            ctx.borrow_mut().start_audio();
            Ok(())
        }
    })?;

    add_event_listener(&window, "keydown", {
        let ctx = ctx.clone();
        move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            ctx.borrow_mut().runner.press(button);
            Ok(())
        }
    })?;

    add_event_listener(&window, "keyup", {
        let ctx = ctx.clone();
        move |event: KeyboardEvent| {
            let button = keycode_binding(&event.code());
            ctx.borrow_mut().runner.release(button);
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
                ctx.borrow_mut().runner.press(*button);
                Ok(())
            }
        })?;

        add_event_listener(&element, "pointerout", {
            let ctx = ctx.clone();
            move |_: PointerEvent| {
                ctx.borrow_mut().runner.release(*button);
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
        move |_: web_sys::Event| {
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

    *g.borrow_mut() = Some(closure({
        let ctx = ctx.clone();
        move |_| {
            request_animation_frame(f.borrow().as_ref().unwrap())?;

            let mut ctx = ctx.borrow_mut();

            for event in ctx.runner.events() {
                match event {
                    Event::RamChanged(ram) => {
                        save_state(ctx.rom_hash, &ram)?;
                    }
                }
            }

            // ImageData doesn't let you pass in shared memory.
            // All the WASM memory is in a SharedArrayBuffer, so we have to copy it to the JS heap
            let buffer = ctx.front_buffer.read_buffer();
            let memory = wasm_bindgen::memory();
            let memory: &WebAssembly::Memory = memory.unchecked_ref();
            let shared_view = Uint8ClampedArray::new_with_byte_offset_and_length(
                &memory.buffer(),
                buffer.as_ptr() as u32,
                buffer.len() as u32,
            );
            let copied = Uint8ClampedArray::new(&shared_view);

            let image_data = ImageData::new_with_js_u8_clamped_array_and_sh(
                &copied,
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

struct NesContext {
    front_buffer: FrontBuffer,
    audio: Option<AudioContext>,
    runner: NESRunner,
    rom_hash: u64,
}

impl NesContext {
    async fn new() -> Result<Self> {
        let (runner, front_buffer, mut audio_source) = NESRunner::new();

        let audio = wasm_audio(Box::new(move |buf| {
            audio_source.read(buf);
            true
        }))
        .await?;

        let mut this = NesContext {
            front_buffer,
            audio: Some(audio),
            runner,
            rom_hash: 0,
        };
        this.set_paused_from_visibility()?;
        Ok(this)
    }

    fn set_rom(&mut self, rom: &[u8]) -> Result<()> {
        let ines = INes::read(rom)?;
        let mut cartridge = ines.into_cartridge();

        let mut rom_hasher = DefaultHasher::new();
        rom.hash(&mut rom_hasher);
        self.rom_hash = rom_hasher.finish();

        if let Some(ram) = load_state(self.rom_hash)? {
            cartridge.set_ram(&ram);
        }

        self.runner.load_cartridge(cartridge);

        Ok(())
    }

    fn set_paused_from_visibility(&mut self) -> Result<()> {
        match document()?.visibility_state() {
            VisibilityState::Hidden => self.runner.pause(),
            VisibilityState::Visible => self.runner.resume(),
            state => bail!("Unrecognised visibility state: {:?}", state),
        };
        Ok(())
    }

    fn start_audio(&mut self) {
        let Some(audio) = self.audio.take() else {
            return;
        };

        async fn resume(audio: AudioContext) -> Result<()> {
            audio.resume().anyhow()?.await.anyhow()?;
            Ok(())
        }

        spawn_local(async {
            if let Err(e) = resume(audio).await {
                log::error!("{}", e);
            }
        });
    }
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

fn document() -> Result<Document> {
    window()?.document().context("DOM not found")
}

fn canvas_context() -> Result<CanvasRenderingContext2d> {
    let canvas = document()?
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

            ctx.borrow_mut().set_rom(&rom)?;
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

fn save_state(rom_hash: u64, ram: &[u8]) -> Result<()> {
    let key = state_key(rom_hash);
    let value = BASE64_STANDARD.encode(ram);
    local_storage()?.set_item(&key, &value).anyhow()?;
    Ok(())
}

fn load_state(rom_hash: u64) -> Result<Option<Vec<u8>>> {
    let key = state_key(rom_hash);
    let value = match local_storage()?.get_item(&key).anyhow()? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(BASE64_STANDARD.decode(value)?))
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
