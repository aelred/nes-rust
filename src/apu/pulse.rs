use super::LENGTH_COUNTER_TABLE;
use bitflags::bitflags;

use super::Length;

use super::envelope::Envelope;

#[derive(Default)]
// A 'pulse wave' is a rectangular wave (alternating from high to low).
pub struct PulseGenerator {
    enabled: bool,
    // `timer` starts at `timer_initial` and counts down to 0.
    // When it reaches 0, it is reloaded with `timer_initial` and `sequencer` is incremented.
    // A lower `timer_initial` value results in a higher frequency.
    timer_initial: u16,
    timer: u16,
    // The index into the waveform.
    sequencer: u8,
    duty_cycle: u8,
    length_counter: u8,
    length_counter_halt: bool,
    envelope: Envelope,
}

impl PulseGenerator {
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length_counter = 0;
        }
        self.enabled = enabled;
    }

    pub fn write_flags(&mut self, value: u8) {
        let flags = PulseFlags::from_bits_truncate(value);
        self.duty_cycle = flags.bits() >> 6;
        self.length_counter_halt = flags.contains(PulseFlags::LENGTH_COUNTER_HALT);
        self.envelope
            .set_constant_volume(flags.contains(PulseFlags::CONSTANT_VOLUME));
        self.envelope
            .set_volume((flags & PulseFlags::VOLUME).bits());
    }

    pub fn write_timer(&mut self, value: u8) {
        // Set the low bits of the timer
        self.timer_initial = (self.timer_initial & 0xFF00) | value as u16;
    }

    pub fn write_length(&mut self, value: u8) {
        // Set the high bits of the timer
        let value = Length::from_bits_truncate(value);
        let timer_high = (value & Length::TIMER_HIGH).bits();
        self.timer_initial = (self.timer_initial & 0x00FF) | ((timer_high as u16) << 8);
        let length_index = (value & Length::LENGTH_COUNTER).bits() >> 3;

        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[length_index as usize];
        }

        self.sequencer = 0;
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
        let waveform = PULSE_DUTY_WAVEFORM[self.duty_cycle as usize];
        let value = (waveform.rotate_right(self.sequencer as u32) & 0b1) * volume * playing as u8;

        if self.timer == 0 {
            self.timer = self.timer_initial;
            self.sequencer = self.sequencer.wrapping_add(1);
        } else {
            self.timer -= 1;
        }

        value
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    struct PulseFlags: u8 {
        const DUTY                = 0b1100_0000;
        const LENGTH_COUNTER_HALT = 0b0010_0000;
        const CONSTANT_VOLUME     = 0b0001_0000;
        const VOLUME              = 0b0000_1111;
    }
}

const PULSE_DUTY_WAVEFORM: [u8; 4] = [
    0b00000010, // 12.5% duty cycle
    0b00000110, // 25% duty cycle
    0b00011110, // 50% duty cycle
    0b11111001, // 25% negated duty cycle
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_generator_produces_rectangle_wave() {
        let mut pulse = PulseGenerator {
            enabled: true,
            timer_initial: 8,
            timer: 8,
            sequencer: 0,
            length_counter: 5,
            length_counter_halt: false,
            // Set duty to 25% and volume goes up to 11
            duty_cycle: 1,
            envelope: Envelope {
                constant_volume: true,
                looping: false,
                start: false,
                divider: 0,
                decay_level: 0,
                volume: 11,
            },
        };

        // Get two periods of the waveform
        let wave: Vec<u8> = std::iter::repeat_with(|| pulse.tick())
            .take(9 * 16)
            .collect();

        // Each part of wave is repeated `timer + 1 = 9` times
        assert_eq!(
            wave,
            [
                vec![0; 9],
                vec![11; 2 * 9],
                vec![0; 6 * 9],
                vec![11; 2 * 9],
                vec![0; 5 * 9]
            ]
            .concat()
        );
    }
}
