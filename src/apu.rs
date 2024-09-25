//! Emulates the APU (audio processing unit)
use bitflags::bitflags;

#[derive(Default)]
pub struct APU {
    pulse_1: PulseGenerator,
    pulse_2: PulseGenerator,
    // APU can run in two "modes", which affect timing and interrupts
    mode_toggle: bool,
    cycles: u16,
}

impl APU {
    pub fn tick(&mut self) -> u8 {
        let pulse_1 = self.pulse_1.tick();
        let pulse_2 = self.pulse_2.tick();

        let cycles = self.cycles;
        self.cycles += 1;

        match (self.mode_toggle, cycles) {
            (_, 3728) | (_, 11185) => {
                self.pulse_1.envelope.clock();
                self.pulse_2.envelope.clock();
            }
            (_, 7456) | (false, 14914) | (true, 18640) => {
                self.pulse_1.envelope.clock();
                self.pulse_2.envelope.clock();
                self.pulse_1.clock_length_counter();
                self.pulse_2.clock_length_counter();
            }
            (false, 14915) | (true, 18641) => {
                self.cycles = 0;
            }
            _ => {}
        }

        pulse_1 + pulse_2
    }

    pub fn write_pulse_1_flags(&mut self, value: u8) {
        self.pulse_1.write_flags(value);
    }

    pub fn write_pulse_1_timer(&mut self, value: u8) {
        self.pulse_1.write_timer(value);
    }

    pub fn write_pulse_1_length(&mut self, value: u8) {
        self.pulse_1.write_length(value);
    }

    pub fn write_pulse_2_flags(&mut self, value: u8) {
        self.pulse_2.write_flags(value);
    }

    pub fn write_pulse_2_timer(&mut self, value: u8) {
        self.pulse_2.write_timer(value);
    }

    pub fn write_pulse_2_length(&mut self, value: u8) {
        self.pulse_2.write_length(value);
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        let value = FrameCounter::from_bits_truncate(value);
        self.mode_toggle = value.contains(FrameCounter::MODE);
    }

    pub fn read_status(&mut self) -> u8 {
        let mut status = Status::empty();
        status.set(Status::PULSE_1, !self.pulse_1.halted());
        status.set(Status::PULSE_2, !self.pulse_2.halted());
        status.bits()
    }

    pub fn write_status(&mut self, value: u8) {
        let status = Status::from_bits_truncate(value);
        self.pulse_1.set_enabled(status.contains(Status::PULSE_1));
        self.pulse_2.set_enabled(status.contains(Status::PULSE_2));
    }
}

#[derive(Default)]
// A 'pulse wave' is a rectangular wave (alternating from high to low).
struct PulseGenerator {
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
    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length_counter = 0;
        }
        self.enabled = enabled;
    }

    fn write_flags(&mut self, value: u8) {
        let flags = PulseFlags::from_bits_truncate(value);
        self.duty_cycle = flags.bits() >> 6;
        self.length_counter_halt = flags.contains(PulseFlags::LENGTH_COUNTER_HALT);
        self.envelope.constant_volume = flags.contains(PulseFlags::CONSTANT_VOLUME);
        self.envelope.volume = (flags & PulseFlags::VOLUME).bits();
    }

    fn write_timer(&mut self, value: u8) {
        // Set the low bits of the timer
        self.timer_initial = (self.timer_initial & 0xFF00) | value as u16;
    }

    fn write_length(&mut self, value: u8) {
        // Set the high bits of the timer
        let value = Length::from_bits_truncate(value);
        let timer_high = (value & Length::TIMER_HIGH).bits();
        self.timer_initial = (self.timer_initial & 0x00FF) | ((timer_high as u16) << 8);
        let length_index = (value & Length::LENGTH_COUNTER).bits() >> 3;

        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[length_index as usize];
        }

        self.sequencer = 0;
        self.envelope.start = true;
    }

    fn halted(&self) -> bool {
        self.length_counter == 0
    }

    // Low-frequency clock to stop sound after a certain time
    fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
        }
    }

    // High-frequency tick to control waveform generation
    fn tick(&mut self) -> u8 {
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

#[derive(Default)]
// An envelope changes a sound's volume over time.
// In the NES APU, it can set a constant volume or a decay.
struct Envelope {
    constant_volume: bool,
    looping: bool,
    start: bool,
    divider: u8,
    decay_level: u8,
    volume: u8,
}

impl Envelope {
    fn clock(&mut self) {
        if !self.start {
            self.clock_divider();
        } else {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.volume;
        }
    }

    fn clock_divider(&mut self) {
        if self.divider == 0 {
            self.divider = self.volume;
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.looping {
                self.decay_level = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    fn volume(&self) -> u8 {
        if self.constant_volume {
            self.volume
        } else {
            self.decay_level
        }
    }
}

bitflags! {
    struct Status: u8 {
        const PULSE_1         = 0b0000_0001;
        const PULSE_2         = 0b0000_0010;
        const TRIANGLE        = 0b0000_0100;
        const NOISE           = 0b0000_1000;
        const DMC             = 0b0001_0000;
        const FRAME_INTERRUPT = 0b1000_0000;
        const DMC_INTERRUPT   = 0b1000_0000;
    }

    #[derive(Copy, Clone)]
    struct PulseFlags: u8 {
        const DUTY                = 0b1100_0000;
        const LENGTH_COUNTER_HALT = 0b0010_0000;
        const CONSTANT_VOLUME     = 0b0001_0000;
        const VOLUME              = 0b0000_1111;
    }

    #[derive(Copy, Clone)]
    struct Length: u8 {
        const LENGTH_COUNTER = 0b1111_1000;
        const TIMER_HIGH     = 0b0000_0111;
    }

    struct FrameCounter: u8 {
        const MODE        = 0b1000_0000;
        const IRQ_INHIBIT = 0b0100_0000;
    }
}

const PULSE_DUTY_WAVEFORM: [u8; 4] = [
    0b00000010, // 12.5% duty cycle
    0b00000110, // 25% duty cycle
    0b00011110, // 50% duty cycle
    0b11111001, // 25% negated duty cycle
];

// I swear, there's a pattern here:
// https://www.nesdev.org/wiki/APU_Length_Counter
#[cfg_attr(any(), rustfmt::skip)]
const LENGTH_COUNTER_TABLE: [u8; 32] = [
    //⬇ Lengths for 90bpm    ⬇ Linearly increasing lengths
     10, /* semiquaver */     254, 
     20, /* quaver */           2, 
     40, /* crotchet */         4, 
     80, /* minim */            6, 
    160, /* semibreve */        8, 
     60, /* dot. crotchet */   10,
     14, /* trip. quaver */    12, 
     26, /* trip. crotchet */  14, 
    //⬇ Lengths for 75bpm
     12, /* semiquaver */      16, 
     24, /* quaver */          18, 
     48, /* crotchet */        20, 
     96, /* minim */           22,
    192, /* semibreve */       24, 
     72, /* dot. crotchet */   26, 
     16, /* trip. quaver */    28, 
     32, /* trip. crotchet */  30,
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
