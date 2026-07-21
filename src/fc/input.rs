use log::debug;

#[derive(Clone, Copy, Default)]
pub struct StandardControllerState {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub struct Controller {
    controller_latch: bool,
    expansion_latch: u8,
    joy1: u8,
    joy2: u8,
    joy1_tmp: u8,
    joy2_tmp: u8,
}

macro_rules! define_read_fn {
    ($fn_name: ident, $joy_num: ident, $num: expr) => {
        /// Function for reading joypad input ($joy_num)
        pub fn $fn_name(&mut self) -> u8 {
            if self.controller_latch {
                // Are these two statements really needed..?
                self.joy1 = self.joy1_tmp;
                self.joy2 = self.joy2_tmp;

                // Whenever latched it always returns the first bit
                return self.$joy_num & 1
            }

            // TODO: bits 5-7 are open bus
            let val = self.$joy_num & 1;
            self.$joy_num >>= 1;
            val
        }
    };
}

macro_rules! define_read_no_sideeffect_fn {
    ($fn_name: ident, $joy_num: ident) => {
        /// Function for reading joypad input without any side effects
        pub fn $fn_name(&self) -> u8 {
            (!self.$joy_num) & 1
        }
    };
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            controller_latch: false,
            expansion_latch: 0x00,
            joy1: 0x00,
            joy2: 0x00,
            joy1_tmp: 0x00,
            joy2_tmp: 0x00,
        }
    }

    pub fn update_from_controller_state(&mut self, joy1_state: StandardControllerState, joy2_state: StandardControllerState) {
        self.joy1_tmp = (joy1_state.a as u8)
                      | (joy1_state.b as u8) << 1
                      | (joy1_state.select as u8) << 2
                      | (joy1_state.start as u8) << 3
                      | (joy1_state.up as u8) << 4
                      | (joy1_state.down as u8) << 5
                      | (joy1_state.left as u8) << 6
                      | (joy1_state.right as u8) << 7;

        if self.joy1_tmp & 0b11000000 == 0b11000000 {
            // Forbid left + right input
            self.joy1_tmp &= 0b111111;
        }
        if self.joy1_tmp & 0b00110000 == 0b00110000 {
            // Forbid up + down input
            self.joy1_tmp &= 0b11001111;
        }

        debug!("wrote {:02x} to joy1_tmp", self.joy1_tmp);

        self.joy2_tmp = (joy2_state.a as u8)
                      | (joy2_state.b as u8) << 1
                      | (joy2_state.select as u8) << 2
                      | (joy2_state.start as u8) << 3
                      | (joy2_state.up as u8) << 4
                      | (joy2_state.down as u8) << 5
                      | (joy2_state.left as u8) << 6
                      | (joy2_state.right as u8) << 7;
        if self.joy2_tmp & 0b11000000 == 0b11000000 {
            // Forbid left + right input
            self.joy2_tmp &= 0b111111;
        }
        if self.joy2_tmp & 0b00110000 == 0b00110000 {
            // Forbid up + down input
            self.joy2_tmp &= 0b11001111;
        }
        debug!("wrote {:02x} to joy2_tmp", self.joy2_tmp);
    }

    define_read_fn!(read_joy1, joy1, 1);
    define_read_fn!(read_joy2, joy2, 2);

    define_read_no_sideeffect_fn!(read_joy1_no_sideeffect, joy1);
    define_read_no_sideeffect_fn!(read_joy2_no_sideeffect, joy2);

    pub fn write(&mut self, val: u8) -> () {
        // TODO: check
        if val & 1 == 0 {
            self.controller_latch = false;
        } else {
            self.joy1 = self.joy1_tmp;
            self.joy2 = self.joy2_tmp;
            self.controller_latch = true;
        }

        // TODO: expansion latch
        if val & 0b110 != 0 {
            self.expansion_latch = (val & 0b110) >> 1;
        } else {
            self.expansion_latch = 0;
        }

        debug!("wrote {val} to joy");
    }

}
