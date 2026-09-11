use log::error;

use crate::{MOTOR_COMMANDS_SIGNAL, motor::MotorsPower};

pub const NORMAL_SPEED: i16 = 100;
const SLOW_SPEED: i16 = 5;
const FAST_SPEED: i16 = 120;

pub enum Event {
    UserCommand(char),
    HeadingUpdated(f32),
}

#[deprecated(note = "Deprecated in favour of control station")]
pub struct MobileAppCommnadsController {
    is_heading_alignment_mode: bool,
    current_speed: i16,
}

impl MobileAppCommnadsController {
    pub fn new() -> Self {
        Self {
            is_heading_alignment_mode: false,
            current_speed: NORMAL_SPEED,
        }
    }
    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::UserCommand(controller_command) => {
                // non-motor commands
                match controller_command {
                    // accell
                    'a' => {
                        self.current_speed = (self.current_speed + 5).clamp(0, 255);
                    }
                    // decel
                    'd' => {
                        self.current_speed = (self.current_speed - 5).clamp(0, 255);
                    }
                    // auto carry - remmapped tp heading align mode
                    'H' => {
                        self.is_heading_alignment_mode = true;
                    }
                    // anti drop - remmapped tp heading align mode disabled
                    'G' => self.is_heading_alignment_mode = false,
                    _ => {
                        MOTOR_COMMANDS_SIGNAL.signal(self.to_motors_power(controller_command));
                    }
                }
            }
            Event::HeadingUpdated(heading) => {
                // if self.is_heading_alignment_mode {
                if heading.to_degrees() < 80.0 {
                    MOTOR_COMMANDS_SIGNAL.signal(MotorsPower::new(FAST_SPEED, SLOW_SPEED));
                } else if heading.to_degrees() > 100.0 {
                    MOTOR_COMMANDS_SIGNAL.signal(MotorsPower::new(SLOW_SPEED, FAST_SPEED));
                } else {
                    MOTOR_COMMANDS_SIGNAL.signal(MotorsPower::new(SLOW_SPEED, SLOW_SPEED));
                }
                // }
            }
        }
    }

    fn to_motors_power(&self, command: char) -> MotorsPower {
        match command {
            // left controll
            'F' => MotorsPower::new(self.current_speed, self.current_speed),
            'B' => MotorsPower::new(-self.current_speed, -self.current_speed),
            'L' => MotorsPower::new(-self.current_speed, self.current_speed),
            'R' => MotorsPower::new(self.current_speed, -self.current_speed),
            // accell
            // decel
            // auto carry
            // anti drop
            'a' | 'd' | 'H' | 'G' => {
                error!("Shouldn't be handled here {}", command);
                MotorsPower::stop()
            }
            // trace
            'X' => MotorsPower::stop(),
            // avoidance
            'Y' => MotorsPower::stop(),
            // following
            'U' => MotorsPower::stop(),
            'S' => {
                // left controll up
                MotorsPower::stop()
            }

            // right controll
            'f' => MotorsPower::new(NORMAL_SPEED, NORMAL_SPEED),
            'b' => MotorsPower::new(-NORMAL_SPEED, -NORMAL_SPEED),
            'l' => MotorsPower::new(-NORMAL_SPEED, NORMAL_SPEED),
            'r' => MotorsPower::new(NORMAL_SPEED, -NORMAL_SPEED),
            // open
            'Q' => MotorsPower::new(255, 255),
            // close
            'E' => MotorsPower::new(5, 5),
            's' => {
                // right controll up
                MotorsPower::stop()
            }
            _ => {
                error!("Unknown command {}", command);
                MotorsPower::stop()
            }
        }
    }
}
