use crate::audio::{audio_pipeline, AudioSource};
use crate::video::{display_triple_buffer, FrontBuffer};
use crate::{Buttons, Cartridge, NES, NES_FREQ};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use web_time::{Duration, Instant};

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
    let start = Instant::now();
    let mut cycles: u64 = 0;
    let mut paused = true;

    // Target CPU cycles per loop before sleeping and checking events.
    // Should be small enough to not fill the audio buffer, but too small adds overhead.
    const CYCLES_PER_LOOP: u64 = 5000;

    loop {
        let mut loop_cycles = 0;
        if paused {
            // Don't tick CPU if paused, pretend we ran cycles to sleep a reasonable time.
            loop_cycles = CYCLES_PER_LOOP;
        } else {
            while loop_cycles < CYCLES_PER_LOOP {
                loop_cycles += nes.tick() as u64;
            }
        }
        cycles += loop_cycles;

        let expected_time = Duration::from_secs_f64(cycles as f64 / NES_FREQ);
        let actual_time = start.elapsed();
        if actual_time < expected_time {
            thread::sleep(expected_time - actual_time);
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
                    return;
                }
            }
        }

        if let Some(ram) = nes.cpu.memory().prg().changed_ram() {
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
