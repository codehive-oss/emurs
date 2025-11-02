pub const CONTROLLER_BUTTON_A: usize = 0;
pub const CONTROLLER_BUTTON_B: usize = 1;
pub const CONTROLLER_BUTTON_SELECT: usize = 2;
pub const CONTROLLER_BUTTON_START: usize = 3;
pub const CONTROLLER_BUTTON_UP: usize = 4;
pub const CONTROLLER_BUTTON_DOWN: usize = 5;
pub const CONTROLLER_BUTTON_LEFT: usize = 6;
pub const CONTROLLER_BUTTON_RIGHT: usize = 7;

pub struct Controller {
    strobe: bool,
    selected_button: usize,
    pub button_states: [bool; 8],
}

impl Controller {
    pub fn new() -> Self {
        Self {
            strobe: false,
            selected_button: 0,
            button_states: [false; 8],
        }
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = (value & 1) == 1;
        if self.strobe {
            self.selected_button = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            return if self.button_states[CONTROLLER_BUTTON_A] {
                1
            } else {
                0
            };
        }
        if self.selected_button >= 8 {
            return 1;
        }
        let value = if self.button_states[self.selected_button] {
            1
        } else {
            0
        };
        self.selected_button += 1;
        value
    }
}
