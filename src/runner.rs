use crate::audio::{audio_pipeline, AudioSource};
use crate::video::{display_triple_buffer, FrontBuffer};
use crate::{Buttons, Cartridge, NES, NES_FREQ};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use web_time::Duration;

pub struct NESRunner {
    commands: Sender<Command>,
    events: Receiver<Event>,
}

impl NESRunner {
    pub fn new() -> (Self, FrontBuffer, AudioSource) {
        let (front_buffer, back_buffer) = display_triple_buffer();
        let (audio_sink, audio_source) = audio_pipeline();
        let (commands, commands_recv) = mpsc::channel();
        let (events_send, events) = mpsc::channel();

        let nes = NES::new(Cartridge::default(), back_buffer, audio_sink);

        wasm_thread::spawn(move || run_nes(nes, commands_recv, events_send));

        let this = Self { commands, events };

        (this, front_buffer, audio_source)
    }

    pub fn resume(&self) {
        self.send(Command::Resume);
    }

    pub fn pause(&self) {
        self.send(Command::Pause);
    }

    pub fn press(&self, buttons: Buttons) {
        self.send(Command::Press(buttons))
    }

    pub fn release(&self, buttons: Buttons) {
        self.send(Command::Release(buttons))
    }

    pub fn load_cartridge(&self, cartridge: Cartridge) {
        self.send(Command::LoadCartridge(cartridge));
    }

    pub fn events(&self) -> impl Iterator<Item = Event> + use<'_> {
        self.events.try_iter()
    }

    fn send(&self, command: Command) {
        self.commands.send(command).expect("unexpectedly stopped");
    }
}

impl Drop for NESRunner {
    fn drop(&mut self) {
        self.send(Command::Stop);
    }
}

fn run_nes(mut nes: NES, commands: Receiver<Command>, events: Sender<Event>) {
    log::info!("Running NES");

    let mut paused = true;

    // Loop/sleep for about one frame between checking events
    const LOOP_DURATION_SECS: f64 = 1.0 / 60.0;
    const CYCLES_PER_LOOP: u64 = (NES_FREQ * LOOP_DURATION_SECS) as u64;

    loop {
        if paused {
            // Don't tick CPU if paused, sleep instead
            std::thread::sleep(Duration::from_secs_f64(LOOP_DURATION_SECS));
        } else {
            // Sleeping is done in the audio buffer sink
            let mut cycles = 0;
            while cycles < CYCLES_PER_LOOP {
                cycles += nes.tick() as u64;
            }
        }

        for command in commands.try_iter() {
            match command {
                Command::Press(buttons) => nes.controller().press(buttons),
                Command::Release(buttons) => nes.controller().release(buttons),
                Command::LoadCartridge(cartridge) => {
                    nes.load_cartridge(cartridge);
                }
                Command::Pause => {
                    paused = true;
                }
                Command::Resume => {
                    paused = false;
                }
                Command::Stop => {
                    log::info!("Stopping NES");
                    return;
                }
            }
        }

        if let Some(ram) = nes.prg.changed_ram() {
            // TODO: ideally don't allocate
            let event = Event::RamChanged(Vec::from(ram));
            let _ = events.send(event);
        }
    }
}

pub enum Command {
    Press(Buttons),
    Release(Buttons),
    LoadCartridge(Cartridge),
    Pause,
    Resume,
    Stop,
}

pub enum Event {
    RamChanged(Vec<u8>),
}
