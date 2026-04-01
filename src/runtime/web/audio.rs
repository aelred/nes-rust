/// Creates an audio worklet which runs in a separate thread.
///
/// Heavily borrowed from the
/// [Wasm audio worklet example](https://wasm-bindgen.github.io/wasm-bindgen/examples/wasm-audio-worklet.html).
use crate::runtime::web::WebResult;
use anyhow::Result;
use js_sys::Array;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions};


/// Start a WASM audio thread running the given callback.
pub fn wasm_audio(
    process: Box<dyn FnMut(&mut [f32]) -> bool>,
) -> AssertUnwindSafe<Pin<Box<dyn Future<Output = Result<AudioContext>>>>> {
    setup_polyfill();

    let process = AssertUnwindSafe(process);
    AssertUnwindSafe(Box::pin(async {
        let ctx = AudioContext::new().anyhow()?;
        prepare_wasm_audio(&ctx).await?;
        let node = wasm_audio_node(&ctx, process.0)?;
        node.connect_with_audio_node(&ctx.destination()).anyhow()?;
        Ok(ctx)
    }))
}

/// Create an audio worklet class that we can instantiate.
async fn prepare_wasm_audio(ctx: &AudioContext) -> Result<()> {
    let mod_url = create_worklet_module_url();
    ctx.audio_worklet()
        .anyhow()?
        .add_module(&mod_url)
        .anyhow()?
        .await
        .anyhow()?;
    Ok(())
}

/// Instantiate the audio worklet with the given callback.
fn wasm_audio_node(
    ctx: &AudioContext,
    process: Box<dyn FnMut(&mut [f32]) -> bool>,
) -> Result<AudioWorkletNode> {
    let options = AudioWorkletNodeOptions::new();
    // Pass the WASM module, its memory and a pointer to a WASM audio processor we just initialised.
    // The worklet creates a NEW instance of the module in another thread, sharing the same memory.
    options.set_processor_options(Some(&Array::of(&[
        wasm_bindgen::module(),
        wasm_bindgen::memory(),
        WasmAudioProcessor(process).pack().into(),
    ])));
    AudioWorkletNode::new_with_options(ctx, "WasmProcessor", &options).anyhow()
}

#[wasm_bindgen]
struct WasmAudioProcessor(Box<dyn FnMut(&mut [f32]) -> bool>);

#[wasm_bindgen]
impl WasmAudioProcessor {
    /// AudioWorkletProcessor method called by audio device on a separate thread.
    pub fn process(&mut self, buffer: &mut [f32]) -> bool {
        self.0(buffer)
    }

    /// Pack into a pointer.
    fn pack(self) -> usize {
        Box::into_raw(Box::new(self)) as usize
    }

    /// Unpack from a pointer.
    pub unsafe fn unpack(pointer: usize) -> Self {
        *Box::from_raw(pointer as *mut _)
    }
}

// This inline JS creates a blob URL for the AudioWorklet processor.
// It computes the main wasm-bindgen module URL by resolving relative to
// this inline module's URL (going up from snippets/.../inline0.js).
// This is necessary because AudioWorklet modules loaded via blob URLs
// cannot use relative imports - they need absolute URLs.
#[wasm_bindgen(inline_js = "
export function create_worklet_module_url() {
    // This inline module is at: snippets/<crate>-<hash>/inline0.js
    // Main module is at: nes-rust.js (2 levels up)
    const bindgenUrl = new URL('../../nes-rust.js', import.meta.url).href;
    return URL.createObjectURL(new Blob([`
        import * as bindgen from '${bindgenUrl}';

        registerProcessor('WasmProcessor', class WasmProcessor extends AudioWorkletProcessor {
            constructor(options) {
                super();
                let [module, memory, handle] = options.processorOptions;
                bindgen.initSync({ module, memory });
                this.processor = bindgen.WasmAudioProcessor.unpack(handle);
            }
            process(inputs, outputs) {
                return this.processor.process(outputs[0][0]);
            }
        });
    `], { type: 'text/javascript' }));
}
")]
extern "C" {
    fn create_worklet_module_url() -> String;
}

// TextDecoder is not available in the audio worklet, because it's not meant to do text processing,
// but in practice it's needed for printing error messages.
#[wasm_bindgen(inline_js = "
if (!globalThis.TextDecoder) {
    // Written in 2013 by Viktor Mukhachev <vic99999@yandex.ru>
    // https://github.com/peter-suggate/wasm-audio-app/blob/0427fdd263271454ef0f580637fa76913d50e0da/public/TextEncoder.js
    globalThis.TextDecoder = class TextDecoder {
        decode(octets) {
            if (!octets) return '';
            var string = '';
            var i = 0;
            while (i < octets.length) {
                var octet = octets[i];
                var bytesNeeded = 0;
                var codePoint = 0;
                if (octet <= 0x7f) {
                    bytesNeeded = 0;
                    codePoint = octet & 0xff;
                } else if (octet <= 0xdf) {
                    bytesNeeded = 1;
                    codePoint = octet & 0x1f;
                } else if (octet <= 0xef) {
                    bytesNeeded = 2;
                    codePoint = octet & 0x0f;
                } else if (octet <= 0xf4) {
                    bytesNeeded = 3;
                    codePoint = octet & 0x07;
                }
                if (octets.length - i - bytesNeeded > 0) {
                    var k = 0;
                    while (k < bytesNeeded) {
                        octet = octets[i + k + 1];
                        codePoint = (codePoint << 6) | (octet & 0x3f);
                        k += 1;
                    }
                } else {
                    codePoint = 0xfffd;
                    bytesNeeded = octets.length - i;
                }
                string += String.fromCodePoint(codePoint);
                i += bytesNeeded + 1;
            }
            return string;
        }
    };
}

// Dummy method to make sure the polyfill is imported.
export function setup_polyfill() { }
")]
extern "C" {
    #[wasm_bindgen]
    fn setup_polyfill();
}
