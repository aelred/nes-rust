use crate::audio::blip_buffer::BlipBuffer;
use crate::audio::ring_buffer::{ring_buffer, RingBufferReader, RingBufferWriter};
use std::sync::atomic::Ordering::Release;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use wasm_thread::JoinHandle;
use Ordering::Acquire;

mod blip_buffer;
mod ring_buffer;

pub const TARGET_AUDIO_FREQ: f64 = 44100.0;
pub const AUDIO_SAMPLE_SIZE: usize = 128;
pub const BUFFER_SIZE: usize = 1024;

/// Create audio pipeline, consisting a sink and source.
pub fn audio_pipeline() -> (AudioSink, AudioSource) {
    let (ring_writer, ring_reader) = ring_buffer(BUFFER_SIZE, 1, AUDIO_SAMPLE_SIZE);

    let audio_sink = AudioSink {
        ring_buffer: ring_writer,
        blip_buffer: BlipBuffer::new(),
    };
    let audio_source = AudioSource {
        buffer: ring_reader,
    };

    (audio_sink, audio_source)
}

/// Sink for audio samples from the NES.
#[derive(Debug)]
pub struct AudioSink {
    blip_buffer: BlipBuffer,
    ring_buffer: RingBufferWriter,
}

impl AudioSink {
    /// Write audio sample.
    ///
    /// Blocks until space is available.
    pub fn write(&mut self, input: f32) {
        debug_assert!(-1.0 <= input && input <= 1.0);

        self.blip_buffer.write(input);

        let Some(output) = self.blip_buffer.next() else {
            return;
        };

        self.ring_buffer.get_mut().0[0] = output;

        while !self.ring_buffer.next(1) {
            // Manage audio timing using the buffer.
            // If it's full, sleep for the time it takes the sink to read one window.
            std::thread::sleep(Duration::from_secs_f64(
                AUDIO_SAMPLE_SIZE as f64 / TARGET_AUDIO_FREQ,
            ));
        }
    }
}

impl Default for AudioSink {
    fn default() -> Self {
        let (audio_sink, audio_source) = audio_pipeline();
        audio_source.silence();
        audio_sink
    }
}

/// Source for audio samples to the audio device.
pub struct AudioSource {
    buffer: RingBufferReader,
}

impl AudioSource {
    /// Read audio samples into the given slice, if possible.
    ///
    /// May quietly fail if the buffer is empty, or the given slice became too large.
    pub fn read(&mut self, out: &mut [f32]) {
        out.fill(0.0);

        // Advance over the _previous_ window.
        // Do this at the start and return silence if it fails.
        if !self.buffer.next(self.buffer.window_size()) {
            return;
        }

        if !self.adjust_window_size(out.len()) {
            return;
        }

        let (buf1, buf2) = self.buffer.get();
        let (out1, out2) = out.split_at_mut(buf1.len());
        out1.copy_from_slice(buf1);
        out2.copy_from_slice(buf2);

        if cfg!(debug_assertions) {
            for v in out {
                assert!(-1.0 <= *v && *v <= 1.0, "Unexpected audio sample: {:?}", v);
            }
        }
    }

    /// Consume all audio samples without playing them
    pub fn silence(self) -> Silencer {
        Silencer::new(self)
    }

    /// Change window size if necessary. Audio devices don't typically do this, but it's possible.
    fn adjust_window_size(&mut self, size: usize) -> bool {
        let window = self.buffer.window_size();
        if size == window {
            return true;
        }

        if !self.buffer.set_window_size(size) {
            log::warn!("Failed to resize audio buffer read window from {window} to {size}");
            return false;
        }

        log::info!("Resized audio buffer read window from {window} to {size}");
        true
    }
}

pub struct Silencer {
    handle: JoinHandle<AudioSource>,
    close: Arc<AtomicBool>,
}

impl Silencer {
    fn new(mut source: AudioSource) -> Self {
        let window_size = source.buffer.window_size();

        let close = Arc::new(AtomicBool::new(false));

        let handle = wasm_thread::spawn({
            let close = close.clone();
            move || {
                while !close.load(Acquire) {
                    let mut reads = 0;
                    while source.buffer.next(window_size) {
                        reads += 1
                    }
                    std::thread::sleep(Duration::from_secs_f64(
                        (reads * window_size) as f64 / TARGET_AUDIO_FREQ,
                    ));
                }
                source
            }
        });

        Self { handle, close }
    }

    pub fn close(self) -> JoinHandle<AudioSource> {
        self.close.store(true, Release);
        self.handle
    }
}
