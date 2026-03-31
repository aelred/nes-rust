use anyhow::Result;
use nes_rust::runtime::sdl;
use nes_rust::runtime::sdl::SdlParams;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Record raw audio bytes to a file
fn main() -> Result<()> {
    let audio_recording_path = Path::new(file!())
        .parent()
        .expect("Failed to get path to audio recording")
        .join("audio-recording.bin");

    let mut callback = Recorder {
        file: BufWriter::new(File::create(audio_recording_path)?),
    };

    sdl::run_with(SdlParams {
        audio_listener: Box::new(move |buffer| callback.write(buffer)),
        ..Default::default()
    })?;

    Ok(())
}

struct Recorder {
    file: BufWriter<File>,
}

impl Recorder {
    fn write(&mut self, buffer: &[f32]) {
        for sample in buffer {
            self.file
                .write_all(&sample.to_le_bytes())
                .expect("Failed to write to recording file");
        }
    }
}
