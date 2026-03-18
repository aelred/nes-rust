use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cell::UnsafeCell;
use std::f32::consts::PI;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use Ordering::{Acquire, Relaxed, Release};

const NES_AUDIO_FREQ: usize = 1_789_773;
const TARGET_AUDIO_FREQ: usize = 44100;
const RATIO: f64 = NES_AUDIO_FREQ as f64 / TARGET_AUDIO_FREQ as f64;

/// A set of experiments with audio playback
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_recording_path = Path::new(file!())
        .parent()
        .expect("Failed to get path to audio recording")
        .join("audio-recording.bin");

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .init();

    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;

    let _window = sdl_context
        .video()?
        .window("nes-rust", 200, 200)
        .position_centered()
        .build()?;

    // Lower target -> higher chance of audio "pops" (consuming audio faster than it can be produced)
    // Higher target -> higher chance of audio "skipping" (producing audio faster than it can be consumed -- sort of, with a big buffer it's very unlikely)
    // Value chosen by experimentation
    const TARGET_LATENCY_SECONDS: f64 = 0.01;
    const TARGET_AVAILABLE_SAMPLES: usize =
        (TARGET_AUDIO_FREQ as f64 * TARGET_LATENCY_SECONDS) as usize;

    let mut buffer_handles = vec![];
    let mut buffer_readers = vec![];

    let mut make_buffer = || {
        let (buffer_handle, buffer_writer, buffer_reader) =
            RingBuffer::new(4096, TARGET_AVAILABLE_SAMPLES);
        buffer_handles.push(buffer_handle);
        buffer_readers.push(buffer_reader);
        buffer_writer
    };

    let mut blip_buffer_pipeline = {
        const STEP_SAMPLE_SIZE: usize = 16;
        const STEP_SUBSAMPLES: usize = 32;
        let step_deltas = (0..STEP_SUBSAMPLES)
            .map(|i| {
                Box::new(band_limited_step_derivative::<STEP_SAMPLE_SIZE>(
                    i as f64 / STEP_SUBSAMPLES as f64,
                )) as Box<[f32]>
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        BlipBuffer::new(make_buffer(), 1.0 / RATIO, step_deltas)
    };

    let mut basic_pipeline = { Resampler::new(RATIO, make_buffer()) };

    let mut filtered_pipeline = {
        fn downsample<const N: usize, S>(ratio: f64, sink: S) -> Filter<Resampler<S>> {
            Filter::new(
                Box::new(windowed_sinc::<N>(0.5 / ratio as f32, 0.0)),
                Resampler::new(ratio, sink),
            )
        }

        // FIXME: this is very slow, ignoring to not interfere with other pipelines
        let _ignored = downsample::<359, _>(
            10.0,
            downsample::<2223, _>(4.0, downsample::<101, _>(RATIO / 40.0, make_buffer())),
        );
        let dummy = Resampler::new(RATIO, make_buffer());
        dummy
    };

    let buffer_idx = Arc::new(AtomicUsize::new(0));
    let _speaker = SDLSpeaker::new(
        &sdl_context,
        buffer_handles,
        buffer_readers,
        buffer_idx.clone(),
    );

    let bytes = std::fs::read(audio_recording_path)?;

    let source = Arc::new(AtomicU8::new(0));
    const SOURCE_RECORDING: u8 = 0;
    const SOURCE_SINE: u8 = 1;
    const SOURCE_SQUARE: u8 = 2;

    let freq = Arc::new(AtomicU32::new(440.0f32.to_bits()));

    let audio_gen_thread = thread::spawn({
        let freq = freq.clone();
        let source = source.clone();
        move || {
            let start = Instant::now();
            let mut ticks = 0;

            for chunk in bytes.as_chunks::<4>().0.into_iter().cycle() {
                let t = ticks as f32 / NES_AUDIO_FREQ as f32;
                let f = f32::from_bits(freq.load(Relaxed));
                let a = 0.1;

                let sample = match source.load(Relaxed) {
                    SOURCE_RECORDING => f32::from_le_bytes(*chunk),
                    SOURCE_SINE => a * 2.0 * (2.0 * PI * t * f).sin(),
                    SOURCE_SQUARE => {
                        a * (2.0 * (2.0 * (f * t).floor() - (2.0 * f * t).floor()) + 1.0)
                    }
                    _ => 0.0,
                };

                blip_buffer_pipeline.receive(sample);
                basic_pipeline.receive(sample);
                filtered_pipeline.receive(sample);

                sleep(start, ticks);

                ticks += 1;
            }
        }
    });

    loop {
        thread::sleep(Duration::from_millis(50));

        if audio_gen_thread.is_finished() {
            audio_gen_thread.join().expect("Audio thread panic");
            return Ok(());
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(());
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Up => {
                        let (Ok(freq) | Err(freq)) = freq.fetch_update(Relaxed, Relaxed, |x| {
                            Some((f32::from_bits(x) * 1.0595).to_bits())
                        });
                        dbg!("Freq up", f32::from_bits(freq));
                    }
                    Keycode::Down => {
                        let (Ok(freq) | Err(freq)) = freq.fetch_update(Relaxed, Relaxed, |x| {
                            Some((f32::from_bits(x) / 1.0595).to_bits())
                        });
                        dbg!("Freq down", f32::from_bits(freq));
                    }
                    Keycode::Left => {
                        source.fetch_sub(1, Relaxed);
                    }
                    Keycode::Right => {
                        source.fetch_add(1, Relaxed);
                    }
                    Keycode::Num1 => {
                        buffer_idx.store(0, Relaxed);
                    }
                    Keycode::Num2 => {
                        buffer_idx.store(1, Relaxed);
                    }
                    Keycode::Num3 => {
                        buffer_idx.store(2, Relaxed);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn sleep(start: Instant, ticks: u32) {
    let tick_duration = Duration::from_secs_f64(1.0 / NES_AUDIO_FREQ as f64);
    let expected_elapsed = tick_duration * ticks;
    let actual_elapsed = Instant::now() - start;
    if actual_elapsed < expected_elapsed {
        thread::sleep(expected_elapsed - actual_elapsed);
    }
}

trait Sink {
    fn receive(&mut self, value: f32);

    fn adjust_rate(&self) -> f64;
}

fn windowed_sinc<const N: usize>(cutoff_freq: f32, phase: f32) -> [f32; N] {
    let offset = (N as f32 - 1.0) / 2.0 + phase;

    let mut kernel = std::array::from_fn(|i| sinc(i as f32 - offset, cutoff_freq));

    window_kernel(&mut kernel);
    normalize_kernel(&mut kernel);

    kernel
}

fn window_kernel(kernel: &mut [f32]) {
    // Apply Blackman window to kernel
    let m = kernel.len() as f32 - 1.0;
    for (i, value) in kernel.iter_mut().enumerate() {
        let i = i as f32;
        *value *= blackman(i / m);
    }
}

fn blackman(x: f32) -> f32 {
    0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos()
}

fn normalize_kernel(kernel: &mut [f32]) {
    let sum: f32 = kernel.iter().sum();
    for value in kernel.iter_mut() {
        *value /= sum;
    }
}

fn sinc(x: f32, cutoff_freq: f32) -> f32 {
    if x == 0.0 {
        2.0 * PI * cutoff_freq
    } else {
        // Some say divide by x, some by x * PI - we normalise so it doesn't matter
        (2.0 * PI * cutoff_freq * x).sin() / x
    }
}

struct Filter<S> {
    cursor: usize,
    buffer: Box<[f32]>,
    kernel: Box<[f32]>,
    sink: S,
}

impl<S> Filter<S> {
    fn new(kernel: Box<[f32]>, sink: S) -> Self {
        let length = kernel.len();
        Self {
            cursor: 0,
            buffer: vec![0.0; length].into_boxed_slice(),
            kernel,
            sink,
        }
    }
}

impl<S: Sink> Sink for Filter<S> {
    fn receive(&mut self, wave: f32) {
        self.buffer[self.cursor % self.buffer.len()] = wave;

        let mut output = 0.0;
        for i in 0..self.buffer.len() {
            output += self.buffer[(self.cursor + i) % self.buffer.len()] * self.kernel[i];
        }
        self.sink.receive(output);

        self.cursor += 1;
    }

    fn adjust_rate(&self) -> f64 {
        self.sink.adjust_rate()
    }
}

struct Resampler<S> {
    /// Ratio of input samples to output samples
    ratio: f64,
    counter: f64,
    sink: S,
}

impl<S> Resampler<S> {
    fn new(ratio: f64, sink: S) -> Self {
        Self {
            ratio,
            counter: 0.0,
            sink,
        }
    }
}

impl<S: Sink> Sink for Resampler<S> {
    fn receive(&mut self, value: f32) {
        // Naive resampling
        while self.counter <= 0.0 {
            self.sink.receive(value);
            self.counter += self.ratio / self.sink.adjust_rate();
        }
        self.counter -= 1.0;
    }

    fn adjust_rate(&self) -> f64 {
        1.0
    }
}

/// Calculates the derivative of a band-limited step.
/// - A step = a wave that jumps from 0 to 1 (rising edge of a square wave)
/// - Band-limited - constructed of a limited number of harmonics up to a certain cut-off frequency
/// - Derivative - taking the difference between each value
///
/// This happens to come to the same thing as a sinc function.
///
/// Applies a window to smooth out the edges.
fn band_limited_step_derivative<const N: usize>(phase: f64) -> [f32; N] {
    // Cut-off must be at most Nyquist (0.5)
    // Lower values will better suppress high frequencies
    windowed_sinc(0.45, phase as f32)
}

struct BlipBuffer {
    buffer: RingBufferWriter,
    ratio: f64,
    step_deltas: Box<[Box<[f32]>]>,
    last_value: f32,
    phase: f64,
    running_sum: f32,
}

impl BlipBuffer {
    fn new(buffer: RingBufferWriter, ratio: f64, step_deltas: Box<[Box<[f32]>]>) -> Self {
        assert!(ratio <= 1.0);
        Self {
            buffer,
            ratio,
            step_deltas,
            last_value: 0.0,
            phase: 0.0,
            running_sum: 0.0,
        }
    }
}

impl Sink for BlipBuffer {
    fn receive(&mut self, value: f32) {
        let delta = value - self.last_value;
        self.last_value = value;

        self.phase += self.ratio * self.buffer.adjust_rate();

        while self.phase >= 1.0 {
            self.buffer.advance(&mut self.running_sum);
            self.phase -= 1.0;
        }

        if delta != 0.0 {
            let phase_index = (self.phase * self.step_deltas.len() as f64).floor() as usize;
            let step_delta = &self.step_deltas[phase_index];
            self.buffer.add(step_delta, delta);
        }
    }

    fn adjust_rate(&self) -> f64 {
        1.0
    }
}

struct RingBuffer {
    capacity: usize,
    buffer: Box<[UnsafeCell<f32>]>,
    read_cursor: AtomicUsize,
    write_cursor: AtomicUsize,
    adjust_rate: AtomicU64,
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    fn new(
        capacity: usize,
        target_available_samples: usize,
    ) -> (RingBufferHandle, RingBufferWriter, RingBufferReader) {
        let buffer = (0..capacity)
            .map(|_| UnsafeCell::new(0.0f32))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let buffer = Arc::new(Self {
            buffer,
            capacity,
            read_cursor: AtomicUsize::new(0),
            write_cursor: AtomicUsize::new(capacity / 2),
            adjust_rate: AtomicU64::new(1.0f64.to_bits()),
        });

        let closed = Arc::new(AtomicBool::new(false));

        let handle = RingBufferHandle {
            closed: closed.clone(),
        };

        let writer = RingBufferWriter {
            buffer: buffer.clone(),
            closed,
        };

        let reader = RingBufferReader {
            buffer,
            target_available_samples,
        };

        (handle, writer, reader)
    }

    fn adjust_rate(&self) -> f64 {
        f64::from_bits(self.adjust_rate.load(Relaxed))
    }

    fn set_adjust_rate(&self, adjust_rate: f64) {
        self.adjust_rate.store(adjust_rate.to_bits(), Relaxed);
    }
}

struct RingBufferHandle {
    closed: Arc<AtomicBool>,
}

impl Drop for RingBufferHandle {
    fn drop(&mut self) {
        self.closed.store(true, Relaxed);
    }
}

struct RingBufferWriter {
    buffer: Arc<RingBuffer>,
    closed: Arc<AtomicBool>,
}

impl RingBufferWriter {
    /// Add given slice to buffer value-by-value, scaled by given amplitude
    fn add(&mut self, input: &[f32], amplitude: f32) {
        let chunk_size = input.len();
        let capacity = self.buffer.capacity;

        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        let read_cursor = self.buffer.read_cursor.load(Acquire);

        let start = write_cursor % capacity;
        let end_write_cursor = write_cursor + chunk_size;
        let end = end_write_cursor % capacity;

        if end_write_cursor >= read_cursor + self.buffer.capacity {
            if write_cursor > 0 && !self.closed.load(Relaxed) {
                dbg!("Writing too fast!", write_cursor, read_cursor);
            }
            return;
        }

        let buffer = self.buffer.buffer.as_ptr() as *mut f32;

        if end > start {
            let slice = unsafe { std::slice::from_raw_parts_mut(buffer.add(start), chunk_size) };
            for (val, inp) in slice.iter_mut().zip(input.iter()) {
                *val += inp * amplitude;
            }
        } else {
            // Requested slice wraps around
            let (slice1, slice2) = unsafe {
                (
                    std::slice::from_raw_parts_mut(buffer.add(start), capacity - start),
                    std::slice::from_raw_parts_mut(buffer, end),
                )
            };
            for (val, inp) in slice1.iter_mut().chain(slice2.iter_mut()).zip(input.iter()) {
                *val += inp * amplitude;
            }
        }
    }

    fn advance(&mut self, running_sum: &mut f32) {
        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        let read_cursor = self.buffer.read_cursor.load(Acquire);

        if write_cursor >= read_cursor + self.buffer.capacity {
            if write_cursor > 0 && !self.closed.load(Relaxed) {
                dbg!("Writing too fast!", write_cursor, read_cursor);
            }
            return;
        }
        let ptr = self.buffer.buffer[write_cursor % self.buffer.capacity].get();
        let delta = unsafe { ptr.read() };
        *running_sum += delta;
        unsafe {
            ptr.write(*running_sum);
        }
        self.buffer.write_cursor.fetch_add(1, Release);
    }
}

impl Sink for RingBufferWriter {
    fn receive(&mut self, value: f32) {
        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        let read_cursor = self.buffer.read_cursor.load(Acquire);

        if write_cursor >= read_cursor + self.buffer.capacity {
            if write_cursor > 0 && !self.closed.load(Relaxed) {
                dbg!("Writing too fast!", write_cursor, read_cursor);
            }
            return;
        }
        let ptr = self.buffer.buffer[write_cursor % self.buffer.capacity].get();
        unsafe {
            ptr.write(value);
        }
        self.buffer.write_cursor.fetch_add(1, Release);
    }

    fn adjust_rate(&self) -> f64 {
        self.buffer.adjust_rate()
    }
}

struct RingBufferReader {
    buffer: Arc<RingBuffer>,
    target_available_samples: usize,
}

impl RingBufferReader {
    fn read_into(&self, out: &mut [f32]) -> usize {
        let chunk_size = out.len();
        let capacity = self.buffer.capacity;

        let write_cursor = self.buffer.write_cursor.load(Acquire);
        let read_cursor = self.buffer.read_cursor.load(Relaxed);

        let start = read_cursor % capacity;
        let new_read_cursor = read_cursor + chunk_size;
        let end = new_read_cursor % capacity;

        if new_read_cursor > write_cursor {
            if read_cursor > 0 {
                dbg!("Reading too fast!", read_cursor, write_cursor, chunk_size);
            }
            return 0;
        }

        let buffer = self.buffer.buffer.as_ptr() as *mut f32;

        if end > start {
            let slice = unsafe { std::slice::from_raw_parts_mut(buffer.add(start), chunk_size) };
            out.copy_from_slice(slice);
            slice.fill(0.0);
        } else {
            // Requested slice wraps around
            let (slice1, slice2) = unsafe {
                (
                    std::slice::from_raw_parts_mut(buffer.add(start), capacity - start),
                    std::slice::from_raw_parts_mut(buffer, end),
                )
            };
            let (out1, out2) = out.split_at_mut(capacity - start);
            out1.copy_from_slice(slice1);
            out2.copy_from_slice(slice2);
            slice1.fill(0.0);
            slice2.fill(0.0);

            // When it wraps around, adjust data production rate to keep the same target latency.
            // Divide by the capacity in order to smooth the adjustment over a full buffer cycle.
            let available_samples = write_cursor.saturating_sub(new_read_cursor);
            let adjust_rate = 1.0
                + (self.target_available_samples as f64 - available_samples as f64)
                    / capacity as f64;
            // "Just-noticeable difference" of human hearing is about 0.5%, so clamp around that
            let adjust_rate = adjust_rate.clamp(0.9975, 1.0025);
            dbg!(adjust_rate, write_cursor - new_read_cursor);
            self.buffer.set_adjust_rate(adjust_rate);
        }

        self.buffer.read_cursor.store(new_read_cursor, Release);

        chunk_size
    }

    fn skip(&self, count: usize) -> usize {
        let capacity = self.buffer.capacity;

        let write_cursor = self.buffer.write_cursor.load(Acquire);
        let read_cursor = self.buffer.read_cursor.load(Relaxed);

        let start = read_cursor % capacity;
        let new_read_cursor = read_cursor + count;
        let end = new_read_cursor % capacity;

        if new_read_cursor > write_cursor {
            return 0;
        }

        let buffer = self.buffer.buffer.as_ptr() as *mut f32;

        if end > start {
            let slice = unsafe { std::slice::from_raw_parts_mut(buffer.add(start), count) };
            slice.fill(0.0);
        } else {
            // Requested slice wraps around
            let (slice1, slice2) = unsafe {
                (
                    std::slice::from_raw_parts_mut(buffer.add(start), capacity - start),
                    std::slice::from_raw_parts_mut(buffer, end),
                )
            };
            slice1.fill(0.0);
            slice2.fill(0.0);

            // When it wraps around, adjust data production rate to keep the same target latency.
            // Divide by the capacity in order to smooth the adjustment over a full buffer cycle.
            let available_samples = write_cursor.saturating_sub(new_read_cursor);
            let adjust_rate = 1.0
                + (self.target_available_samples as f64 - available_samples as f64)
                    / capacity as f64;
            // "Just-noticeable difference" of human hearing is about 0.5%, so clamp around that
            let adjust_rate = adjust_rate.clamp(0.9975, 1.0025);
            self.buffer.set_adjust_rate(adjust_rate);
        }

        self.buffer.read_cursor.store(new_read_cursor, Release);

        count
    }
}

struct SDLSpeaker {
    // Put the handles before the device so they get dropped first
    _buffer_handles: Vec<RingBufferHandle>,
    _device: AudioDevice<MyAudioCallback>,
}

impl SDLSpeaker {
    fn new(
        sdl_context: &sdl2::Sdl,
        buffer_handles: Vec<RingBufferHandle>,
        buffer_readers: Vec<RingBufferReader>,
        buffer_idx: Arc<AtomicUsize>,
    ) -> Result<Self, String> {
        let audio_subsystem = sdl_context.audio()?;

        let desired_spec = AudioSpecDesired {
            freq: Some(TARGET_AUDIO_FREQ as i32),
            channels: Some(1),
            samples: Some(128),
        };

        let device =
            audio_subsystem.open_playback(None, &desired_spec, |_spec| MyAudioCallback {
                buffers: buffer_readers,
                buffer_idx,
            })?;
        device.resume();

        Ok(Self {
            _buffer_handles: buffer_handles,
            _device: device,
        })
    }
}

struct MyAudioCallback {
    buffers: Vec<RingBufferReader>,
    buffer_idx: Arc<AtomicUsize>,
}

impl AudioCallback for MyAudioCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let i = self.buffer_idx.load(Relaxed);

        // Consume all buffers so they can keep writing even if we don't use them
        for (j, buffer) in self.buffers.iter().enumerate() {
            if i != j {
                buffer.skip(out.len());
            }
        }

        self.buffers[i].read_into(out);
    }
}
