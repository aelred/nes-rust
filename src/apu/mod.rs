//! Emulates the APU (audio processing unit)
use bitflags::bitflags;
use noise::NoiseGenerator;
use pulse::PulseGenerator;
use triangle::TriangleGenerator;

mod envelope;
mod noise;
mod pulse;
mod triangle;

#[derive(Default)]
pub struct APU {
    pulse_1: PulseGenerator,
    pulse_2: PulseGenerator,
    triangle: TriangleGenerator,
    noise: NoiseGenerator,
    // APU can run in two "modes", which affect timing and interrupts
    mode_toggle: bool,
    cycles: u16,
}

impl APU {
    pub fn tick(&mut self) -> f32 {
        let pulse_1 = self.pulse_1.tick();
        let pulse_2 = self.pulse_2.tick();
        let triangle = self.triangle.tick();
        let noise = self.noise.tick();

        let cycles = self.cycles;
        self.cycles += 1;

        match (self.mode_toggle, cycles) {
            (_, 7457) | (_, 22371) => {
                self.pulse_1.clock_envelope();
                self.pulse_2.clock_envelope();
                self.noise.clock_envelope();
                self.triangle.clock_linear_counter();
            }
            (_, 14913) | (false, 29829) | (true, 37281) => {
                self.pulse_1.clock_envelope();
                self.pulse_2.clock_envelope();
                self.noise.clock_envelope();
                self.triangle.clock_linear_counter();
                self.pulse_1.clock_length_counter();
                self.pulse_2.clock_length_counter();
                self.triangle.clock_length_counter();
                self.noise.clock_length_counter();
            }
            (false, 14915) | (true, 37282) => {
                self.cycles = 0;
            }
            _ => {}
        }

        mix(pulse_1, pulse_2, triangle, noise)
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

    pub fn write_triangle_flags(&mut self, value: u8) {
        self.triangle.write_flags(value);
    }

    pub fn write_triangle_timer(&mut self, value: u8) {
        self.triangle.write_timer(value);
    }

    pub fn write_triangle_length(&mut self, value: u8) {
        self.triangle.write_length(value);
    }

    pub fn write_noise_flags(&mut self, value: u8) {
        self.noise.write_flags(value);
    }

    pub fn write_noise_mode(&mut self, value: u8) {
        self.noise.write_mode(value);
    }

    pub fn write_noise_length(&mut self, value: u8) {
        self.noise.write_length(value);
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
        self.triangle.set_enabled(status.contains(Status::TRIANGLE));
        self.noise.set_enabled(status.contains(Status::NOISE));
    }
}

// Mix output channels, produce a value between 0.0 and 1.0
fn mix(pulse_1: u8, pulse_2: u8, triangle: u8, noise: u8) -> f32 {
    let pulse_in = (pulse_1 + pulse_2) as f32;
    let pulse_out = if pulse_in == 0.0 {
        0.0
    } else {
        95.88 / ((8128.0 / pulse_in) + 100.0)
    };

    let tnd_in = (triangle as f32) / 8227.0 + (noise as f32) / 12241.0;
    let tnd_out = if tnd_in == 0.0 {
        0.0
    } else {
        159.79 / (1.0 / tnd_in + 100.0)
    };
    pulse_out + tnd_out
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
    struct Length: u8 {
        const LENGTH_COUNTER = 0b1111_1000;
        const TIMER_HIGH     = 0b0000_0111;
    }

    struct FrameCounter: u8 {
        const MODE        = 0b1000_0000;
        const IRQ_INHIBIT = 0b0100_0000;
    }
}

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
