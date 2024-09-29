use super::LENGTH_COUNTER_TABLE;
use bitflags::bitflags;

use super::envelope::Envelope;

// A pseudo-random noise generator
pub struct NoiseGenerator {
    enabled: bool,
    // `timer` starts at `timer_initial` and counts down to 0.
    // When it reaches 0, it is reloaded with `timer_initial` and `shift_register` is adjusted.
    // A lower `timer_initial` value results in higher frequency noise.
    timer_initial: u16,
    timer: u16,
    mode: bool,
    shift_register: u16,
    length_counter: u8,
    length_counter_halt: bool,
    envelope: Envelope,
}

impl NoiseGenerator {
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length_counter = 0;
        }
        self.enabled = enabled;
    }

    pub fn write_flags(&mut self, value: u8) {
        let flags = NoiseFlags::from_bits_truncate(value);
        self.length_counter_halt = flags.contains(NoiseFlags::LENGTH_COUNTER_HALT);
        self.envelope
            .set_constant_volume(flags.contains(NoiseFlags::CONSTANT_VOLUME));
        self.envelope
            .set_volume((flags & NoiseFlags::VOLUME).bits());
    }

    pub fn write_mode(&mut self, value: u8) {
        let value = Mode::from_bits_truncate(value);
        self.mode = value.contains(Mode::MODE);
        let period = (value & Mode::PERIOD).bits();
        self.timer_initial = TIMER[period as usize];
    }

    pub fn write_length(&mut self, value: u8) {
        // Set the high bits of the timer
        let value = Length::from_bits_truncate(value);
        let length_index = (value & Length::LENGTH_COUNTER).bits() >> 3;

        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[length_index as usize];
        }

        self.envelope.start();
    }

    pub fn halted(&self) -> bool {
        self.length_counter == 0
    }

    // Low-frequency clock to reduce sound over time
    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    // Low-frequency clock to stop sound after a certain time
    pub fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
        }
    }

    // High-frequency tick to control waveform generation
    pub fn tick(&mut self) -> u8 {
        let playing = !self.halted();
        let volume = self.envelope.volume();
        let value = (!self.shift_register & 0b1) as u8 * volume * playing as u8;

        if self.timer == 0 {
            self.timer = self.timer_initial;
            self.clock_shift_register();
        } else {
            self.timer -= 1;
        }

        value
    }

    fn clock_shift_register(&mut self) {
        // Create a pseudo-random bit sequence, by:
        // 1. Choose 'n'th bit to use to adjust the shift register based on the mode
        let bit = if self.mode { 6 } else { 1 };
        // 2. Calculate 'feedback' as the XOR of the first and 'n'th bits.
        let feedback = (self.shift_register & 0b1) ^ ((self.shift_register >> bit) & 0b1);
        // 3. Shift the register right by 1 bit.
        self.shift_register >>= 1;
        // 4. Set the last shift register bit to 'feedback'
        self.shift_register |= feedback << 14;
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self {
            enabled: false,
            timer_initial: 0,
            timer: 0,
            mode: false,
            shift_register: 1,
            length_counter: 0,
            length_counter_halt: false,
            envelope: Default::default(),
        }
    }
}

const TIMER: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

bitflags! {
    #[derive(Copy, Clone)]
    struct NoiseFlags: u8 {
        const LENGTH_COUNTER_HALT = 0b0010_0000;
        const CONSTANT_VOLUME     = 0b0001_0000;
        const VOLUME              = 0b0000_1111;
    }

    #[derive(Copy, Clone)]
    struct Mode: u8 {
        const MODE   = 0b1000_0000;
        const PERIOD = 0b0000_1111;
    }

    #[derive(Copy, Clone)]
    struct Length: u8 {
        const LENGTH_COUNTER = 0b1111_1000;
    }
}
