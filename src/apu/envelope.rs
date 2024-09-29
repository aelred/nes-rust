#[derive(Default)]
// An envelope changes a sound's volume over time.
// In the NES APU, it can set a constant volume or a decay.
pub struct Envelope {
    constant_volume: bool,
    looping: bool,
    start: bool,
    divider: u8,
    decay_level: u8,
    volume: u8,
}

impl Envelope {
    pub fn start(&mut self) {
        self.start = true;
    }

    pub fn set_constant_volume(&mut self, constant_volume: bool) {
        self.constant_volume = constant_volume;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    pub fn clock(&mut self) {
        if !self.start {
            self.clock_divider();
        } else {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.volume;
        }
    }

    pub fn clock_divider(&mut self) {
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

    pub fn volume(&self) -> u8 {
        if self.constant_volume {
            self.volume
        } else {
            self.decay_level
        }
    }
}
