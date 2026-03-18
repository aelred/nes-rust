use nes_rust::runtime::sdl::{SDLDisplay, SDLSpeaker, Sdl};
use nes_rust::NESSpeaker;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Record raw audio bytes to a file
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_recording_path = Path::new(file!())
        .parent()
        .expect("Failed to get path to audio recording")
        .join("audio-recording.bin");

    let sdl_context = sdl2::init()?;
    let display = SDLDisplay::new(&sdl_context)?;
    let speaker = RecordingSpeaker {
        file: BufWriter::new(File::create(audio_recording_path)?),
        speaker: SDLSpeaker::new(&sdl_context)?,
    };

    Sdl::run_with(&sdl_context, display, speaker)?;

    Ok(())
}

struct RecordingSpeaker<S> {
    file: BufWriter<File>,
    speaker: S,
}

impl<S: NESSpeaker> NESSpeaker for RecordingSpeaker<S> {
    fn emit(&mut self, wave: f32) {
        self.file
            .write(&wave.to_le_bytes())
            .expect("Failed to write to recording file");
        self.speaker.emit(wave)
    }
}
