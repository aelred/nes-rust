use super::LENGTH_COUNTER_TABLE;
use bitflags::bitflags;

use super::Length;

#[derive(Default)]
// A 'triangle wave' is a waveform that goes up and down in a triangle shape.
pub struct TriangleGenerator {
    enabled: bool,
    // `timer` starts at `timer_initial` and counts down to 0.
    // When it reaches 0, it is reloaded with `timer_initial` and `sequencer` is incremented.
    // A lower `timer_initial` value results in a higher frequency.
    timer_initial: u16,
    timer: u16,
    // The index into the waveform.
    sequencer: u8,
    length_counter: u8,
    length_counter_halt: bool,
    linear_counter: u8,
    linear_counter_reload: u8,
    linear_counter_reload_flag: bool,
    linear_counter_control: bool,
}

impl TriangleGenerator {
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length_counter = 0;
        }
        self.enabled = enabled;
    }

    pub fn write_flags(&mut self, value: u8) {
        let flags = TriangleFlags::from_bits_truncate(value);
        self.length_counter_halt = flags.contains(TriangleFlags::LENGTH_COUNTER_HALT);
        self.linear_counter_control = flags.contains(TriangleFlags::LINEAR_COUNTER_CONTROL);
        self.linear_counter_reload = (flags & TriangleFlags::LINEAR_COUNTER_RELOAD).bits();
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

        self.linear_counter_reload_flag = true;
    }

    pub fn halted(&self) -> bool {
        self.length_counter == 0
    }

    // Low-frequency clock to stop sound after a certain time
    pub fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
        }
    }

    // Low-frequency clock to stop sound after a certain time, with more fine-grained control
    pub fn clock_linear_counter(&mut self) {
        self.linear_counter = self.linear_counter.saturating_sub(1);
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload;
        }
        if !self.linear_counter_control {
            self.linear_counter_reload_flag = false;
        }
    }

    // High-frequency tick to control waveform generation
    pub fn tick(&mut self) -> u8 {
        let playing = !self.halted();
        let value = TRIANGLE_WAVEFORM[(self.sequencer % 32) as usize] * playing as u8;

        if self.timer == 0 {
            self.timer = self.timer_initial;
            if self.length_counter != 0 && self.linear_counter != 0 {
                self.sequencer = self.sequencer.wrapping_add(1);
            }
        } else {
            self.timer -= 1;
        }

        value
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    struct TriangleFlags: u8 {
        // Linear counter control and length counter halt share a flag
        const LENGTH_COUNTER_HALT    = 0b1000_0000;
        const LINEAR_COUNTER_CONTROL = 0b1000_0000;
        const LINEAR_COUNTER_RELOAD  = 0b0111_1111;
    }
}

const TRIANGLE_WAVEFORM: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, // descending
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // ascending
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_generator_produces_triangle_wave() {
        let mut triangle = TriangleGenerator {
            enabled: true,
            timer_initial: 8,
            timer: 8,
            sequencer: 0,
            length_counter: 5,
            length_counter_halt: false,
            linear_counter: 255,
            linear_counter_reload: 255,
            linear_counter_reload_flag: false,
            linear_counter_control: false,
        };

        // Get two periods of the waveform
        let wave: Vec<u8> = std::iter::repeat_with(|| triangle.tick())
            .take(9 * 64)
            .collect();

        // Each part of wave is repeated `timer + 1 = 9` times
        assert_eq!(
            wave,
            [
                vec![15; 9],
                vec![14; 9],
                vec![13; 9],
                vec![12; 9],
                vec![11; 9],
                vec![10; 9],
                vec![9; 9],
                vec![8; 9],
                vec![7; 9],
                vec![6; 9],
                vec![5; 9],
                vec![4; 9],
                vec![3; 9],
                vec![2; 9],
                vec![1; 9],
                vec![0; 9],
                vec![0; 9],
                vec![1; 9],
                vec![2; 9],
                vec![3; 9],
                vec![4; 9],
                vec![5; 9],
                vec![6; 9],
                vec![7; 9],
                vec![8; 9],
                vec![9; 9],
                vec![10; 9],
                vec![11; 9],
                vec![12; 9],
                vec![13; 9],
                vec![14; 9],
                vec![15; 9]
            ]
            .concat()
            .repeat(2)
        );
    }
}
