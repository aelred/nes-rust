use bitflags::bitflags;

pub trait Input {
    fn read(&mut self) -> u8;
    fn write(&mut self, value: u8);
}

pub struct Controller {
    buttons: Buttons,
    strobe: bool,
    read_cursor: u8,
}

const CURSOR_START: u8 = 0b1000_0000;

impl Controller {
    pub fn press(&mut self, button: Button) {
        self.buttons.insert(button.into());
    }

    pub fn release(&mut self, button: Button) {
        self.buttons.remove(button.into());
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

pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

bitflags! {
    #[derive(Default)]
    struct Buttons: u8 {
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

impl From<Button> for Buttons {
    fn from(button: Button) -> Self {
        match button {
            Button::A => Buttons::A,
            Button::B => Buttons::B,
            Button::Select => Buttons::SELECT,
            Button::Start => Buttons::START,
            Button::Up => Buttons::UP,
            Button::Down => Buttons::DOWN,
            Button::Left => Buttons::LEFT,
            Button::Right => Buttons::RIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_and_releasing_buttons_toggles_fields() {
        let mut controller = Controller::default();
        assert_eq!(controller.buttons.bits(), 0b0000_0000);

        controller.press(Button::A);
        controller.press(Button::B);
        controller.press(Button::Select);
        controller.press(Button::Start);
        assert_eq!(controller.buttons.bits(), 0b1111_0000);

        controller.release(Button::A);
        assert_eq!(controller.buttons.bits(), 0b0111_0000);

        controller.press(Button::Up);
        assert_eq!(controller.buttons.bits(), 0b0111_1000);

        controller.press(Button::Down);
        controller.press(Button::Left);
        controller.press(Button::Right);
        assert_eq!(controller.buttons.bits(), 0b0111_1111);

        controller.release(Button::Up);
        controller.release(Button::Down);
        controller.release(Button::Left);
        controller.release(Button::Right);
        assert_eq!(controller.buttons.bits(), 0b0111_0000);
    }

    #[test]
    fn when_strobe_is_toggled_off_button_status_is_reported() {
        let mut controller = Controller::default();

        controller.buttons = Buttons::from_bits_truncate(0b1001_0110);

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

        controller.press(Button::A);
        assert_eq!(controller.read(), 1);
        assert_eq!(controller.read(), 1);

        controller.release(Button::A);
        assert_eq!(controller.read(), 0);
        assert_eq!(controller.read(), 0);

        controller.press(Button::A);
        controller.write(0);
        assert_eq!(controller.read(), 1);
        assert_eq!(controller.read(), 0);
    }

    #[test]
    fn after_reading_status_subsequent_reads_return_zero() {
        let mut controller = Controller::default();

        controller.buttons = Buttons::from_bits_truncate(0b1001_0110);

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
