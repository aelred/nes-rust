use nes_rust::audio::{audio_pipeline, AudioSource};
use nes_rust::runtime::sdl::{SDLSpeaker, Sdl};
use sdl2::audio::AudioCallback;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Record raw audio bytes to a file
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_recording_path = Path::new(file!())
        .parent()
        .expect("Failed to get path to audio recording")
        .join("audio-recording.bin");

    let (audio_sink, audio_source) = audio_pipeline();
    let sdl_context = sdl2::init()?;
    let callback = RecordingCallback {
        file: BufWriter::new(File::create(audio_recording_path)?),
        audio_source,
    };
    let _sdl_speaker = SDLSpeaker::new(&sdl_context, callback)?;

    Sdl::run_with(&sdl_context, audio_sink)?;

    Ok(())
}

struct RecordingCallback {
    file: BufWriter<File>,
    audio_source: AudioSource,
}

impl AudioCallback for RecordingCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.audio_source.read(out);
        for sample in out {
            self.file
                .write_all(&sample.to_le_bytes())
                .expect("Failed to write to recording file");
        }
    }
}
