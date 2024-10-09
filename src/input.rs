use bitflags::bitflags;

pub trait Input {
    fn read(&mut self) -> u8;
    fn write(&mut self, value: u8);
}

#[derive(Debug)]
pub struct Controller {
    buttons: Buttons,
    strobe: bool,
    read_cursor: u8,
}

const CURSOR_START: u8 = 0b1000_0000;

impl Controller {
    pub fn press(&mut self, buttons: Buttons) {
        self.buttons.insert(buttons);
    }

    pub fn release(&mut self, buttons: Buttons) {
        self.buttons.remove(buttons);
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            buttons: Buttons::default(),
            strobe: false,
            read_cursor: CURSOR_START,
        }
    }
}

impl Input for Controller {
    fn read(&mut self) -> u8 {
        if self.strobe {
            self.read_cursor = CURSOR_START;
        }

        let button_pressed = (self.buttons.bits() & self.read_cursor) != 0;

        if !self.strobe {
            self.read_cursor >>= 1;
        }

        button_pressed.into()
    }

    fn write(&mut self, value: u8) {
        self.strobe = value & 0b1 != 0;

        if self.strobe {
            self.read_cursor = CURSOR_START;
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Copy, Clone)]
    pub struct Buttons: u8 {
        const A      = 0b1000_0000;
        const B      = 0b0100_0000;
        const SELECT = 0b0010_0000;
        const START  = 0b0001_0000;
        const UP     = 0b0000_1000;
        const DOWN   = 0b0000_0100;
        const LEFT   = 0b0000_0010;
        const RIGHT  = 0b0000_0001;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_and_releasing_buttons_toggles_fields() {
        let mut controller = Controller::default();
        assert_eq!(controller.buttons.bits(), 0b0000_0000);

        controller.press(Buttons::A);
        controller.press(Buttons::B);
        controller.press(Buttons::SELECT);
        controller.press(Buttons::START);
        assert_eq!(controller.buttons.bits(), 0b1111_0000);

        controller.release(Buttons::A);
        assert_eq!(controller.buttons.bits(), 0b0111_0000);

        controller.press(Buttons::UP);
        assert_eq!(controller.buttons.bits(), 0b0111_1000);

        controller.press(Buttons::DOWN);
        controller.press(Buttons::LEFT);
        controller.press(Buttons::RIGHT);
        assert_eq!(controller.buttons.bits(), 0b0111_1111);

        controller.release(Buttons::UP);
        controller.release(Buttons::DOWN);
        controller.release(Buttons::LEFT);
        controller.release(Buttons::RIGHT);
        assert_eq!(controller.buttons.bits(), 0b0111_0000);
    }

    #[test]
    fn when_strobe_is_toggled_off_button_status_is_reported() {
        let mut controller = Controller {
            buttons: Buttons::from_bits_truncate(0b1001_0110),
            ..Controller::default()
        };

        controller.write(1);
        controller.write(0);

        // Read bit-by-bit
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);

        controller.write(1);
        controller.write(0);

        // Read bit-by-bit
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b1);
        assert_eq!(controller.read(), 0b0);
    }

    #[test]
    fn while_strobe_is_on_always_report_state_of_button_a() {
        let mut controller = Controller::default();

        controller.write(1);
        assert_eq!(controller.read(), 0);
        assert_eq!(controller.read(), 0);

        controller.press(Buttons::A);
        assert_eq!(controller.read(), 1);
        assert_eq!(controller.read(), 1);

        controller.release(Buttons::A);
        assert_eq!(controller.read(), 0);
        assert_eq!(controller.read(), 0);

        controller.press(Buttons::A);
        controller.write(0);
        assert_eq!(controller.read(), 1);
        assert_eq!(controller.read(), 0);
    }

    #[test]
    fn after_reading_status_subsequent_reads_return_zero() {
        let mut controller = Controller {
            buttons: Buttons::from_bits_truncate(0b1001_0110),
            ..Controller::default()
        };

        controller.write(1);
        controller.write(0);

        for _ in 0..8 {
            controller.read();
        }

        for _ in 0..100 {
            assert_eq!(controller.read(), 0);
        }
    }
}
