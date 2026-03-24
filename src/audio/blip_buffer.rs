use crate::audio::TARGET_AUDIO_FREQ;
use crate::NES_FREQ;
use std::collections::VecDeque;
use std::f32::consts::PI;

const STEP_SAMPLE_SIZE: usize = 16;
const STEP_SUBSAMPLES: usize = 32;

/// Blip buffer, which provides band-limited sound synthesis.
///
/// NES audio frequency is very high (~40x higher than a typical audio device), consists
/// of discrete "steps" and can produce very high frequency sounds.
///
/// Blip buffer can heavily downsample more efficiently than a typical low-pass filter, by taking
/// advantage of the regularity of the synthesised steps, which conveniently look the same when
/// perfectly low-passed and downsampled.
///
/// The basics of the algorithm are described here: http://www.slack.net/~ant/bl-synth/
pub struct BlipBuffer {
    /// Tiny ring buffer which always contains STEP_SAMPLE_SIZE elements
    diff_buffer: VecDeque<f32>,
    /// Pre-calculated step differences which can be added together
    step_diffs: Box<[Box<[f32]>; STEP_SUBSAMPLES]>,
    /// The last input sample, used to calculate differences between input samples
    last_input: f32,
    /// The current phase through the next output sample between 0.0 and 1.0
    phase: f64,
    /// The most recent output sample, calculated as a running sum of differences
    output: f32,
}

impl BlipBuffer {
    pub fn new() -> Self {
        let mut diff_buffer = VecDeque::with_capacity(STEP_SAMPLE_SIZE);
        for _ in 0..STEP_SAMPLE_SIZE {
            diff_buffer.push_back(0.0);
        }

        // Calculate the "step difference" at different phases between 0.0 and 1.0
        let step_diffs = std::array::from_fn(|i| windowed_sinc(i as f32 / STEP_SUBSAMPLES as f32));

        Self {
            diff_buffer,
            step_diffs: Box::new(step_diffs),
            last_input: 0.0,
            phase: 0.0,
            output: 0.0,
        }
    }

    /// Write an input sample into the buffer.
    pub fn write(&mut self, input: f32) {
        // Skip if input hasn't changed
        if input == self.last_input {
            return;
        }

        let diff = input - self.last_input;
        self.last_input = input;

        let phase_index = (self.phase * self.step_diffs.len() as f64).floor() as usize;
        let step_diff = &self.step_diffs[phase_index];

        // Write step difference to buffer, scaled by amplitude of the difference
        let (s1, s2) = self.diff_buffer.as_mut_slices();
        let slice = s1.iter_mut().chain(s2.iter_mut());
        for (val, step) in slice.zip(step_diff) {
            *val += step * diff;
        }
    }

    /// Advance the blip buffer by the NES frequency, should be called for each APU tick.
    ///
    /// Every ~40 calls, returns the next downsampled output value.
    pub fn next(&mut self) -> Option<f32> {
        self.phase += TARGET_AUDIO_FREQ / NES_FREQ;

        // Check if we've advanced to the next output sample
        if self.phase < 1.0 {
            return None;
        }

        let front_diff = self
            .diff_buffer
            .pop_front()
            .expect("blip buffer is never empty");
        self.diff_buffer.push_back(0.0);

        // Maintain a running sum of the output using the differences
        self.output += front_diff;
        self.output = self.output.clamp(-1.0, 1.0);

        self.phase -= 1.0;
        debug_assert!(self.phase >= 0.0 && self.phase < 1.0);

        Some(self.output)
    }
}

/// Calculates a windowed sinc function at the given phase.
///
/// A sinc function turns out to be the derivative of a band-limited step:
/// - A step = a wave that jumps from 0 to 1 (rising edge of a square wave)
/// - Band-limited - constructed of a limited number of harmonics up to a certain cut-off frequency
/// - Derivative - taking the difference between each value
///
/// Applies a window to smooth out the edges.
fn windowed_sinc(phase: f32) -> Box<[f32]> {
    // Cut-off must be at most Nyquist (0.5)
    // Lower values will better suppress high frequencies
    const CUTOFF_FREQ: f32 = 0.45;

    let offset = (STEP_SAMPLE_SIZE - 1) as f32 / 2.0 + phase;

    let mut kernel: Vec<f32> = (0..STEP_SAMPLE_SIZE)
        .map(|i| sinc(i as f32 - offset, CUTOFF_FREQ) * window(i))
        .collect();

    // Normalise kernel to sum to 1
    let sum: f32 = kernel.iter().sum();
    for value in kernel.iter_mut() {
        *value /= sum;
    }

    kernel.into_boxed_slice()
}

/// Blackman window
fn window(n: usize) -> f32 {
    let x = n as f32 / (STEP_SAMPLE_SIZE - 1) as f32;
    0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos()
}

/// Sinc function
fn sinc(x: f32, cutoff_freq: f32) -> f32 {
    if x == 0.0 {
        2.0 * cutoff_freq
    } else {
        (2.0 * PI * cutoff_freq * x).sin() / (x * PI)
    }
}
