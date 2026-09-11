use log::{error, info};

use crate::{MOTOR_COMMANDS_SIGNAL, motor::MotorsPower};

pub const NORMAL_SPEED: i16 = 150;

pub enum Event {
    UserCommand(char),
    HeadingUpdated(f32),
}

pub struct MainController {
    is_heading_alignment_mode: bool,
    current_speed: MotorsPower,
}

impl MainController {
    pub fn new() -> Self {
        Self {
            is_heading_alignment_mode: false,
            current_speed: MotorsPower::stop(),
        }
    }
    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::UserCommand(controller_command) => {
                // non-motor commands
                let motor_signal = match controller_command {
                    // accell
                    'z' => {
                        self.current_speed = MotorsPower::new(
                            (self.current_speed.left + 5).clamp(0, 255),
                            (self.current_speed.right + 5).clamp(0, 255),
                        );
                        Some(self.current_speed.clone())
                    }
                    // decel
                    'x' => {
                        self.current_speed = MotorsPower::new(
                            (self.current_speed.left - 5).clamp(0, 255),
                            (self.current_speed.right - 5).clamp(0, 255),
                        );
                        Some(self.current_speed.clone())
                    }
                    // heading align mode
                    'c' => {
                        self.is_heading_alignment_mode = true;
                        None
                    }
                    // heading align mode disabled
                    'v' => {
                        self.is_heading_alignment_mode = false;
                        None
                    }

                    //forward
                    'w' => Some(self.current_speed.clone()),
                    //backward
                    's' => Some(MotorsPower::new(
                        -self.current_speed.left,
                        -self.current_speed.right,
                    )),
                    //left
                    'a' => Some(MotorsPower::new(
                        increase_by_percent(self.current_speed.left, 0),
                        increase_by_percent(self.current_speed.right, 30),
                    )),
                    //right
                    'd' => Some(MotorsPower::new(
                        increase_by_percent(self.current_speed.left, 20),
                        increase_by_percent(self.current_speed.right, 0),
                    )),
                    //start normal speed
                    'n' => {
                        self.current_speed = MotorsPower::new(NORMAL_SPEED, NORMAL_SPEED);
                        Some(self.current_speed.clone())
                    }
                    _ => {
                        error!("Unknown command {}", controller_command);
                        Some(MotorsPower::stop())
                    }
                };
                if let Some(s) = motor_signal {
                    MOTOR_COMMANDS_SIGNAL.signal(s);
                }
                info!(
                    "Speed is l:{} r:{}",
                    self.current_speed.left, self.current_speed.right
                );
            }
            Event::HeadingUpdated(heading) => {
                if self.is_heading_alignment_mode {
                    let correction = calculate_correction(heading, self.current_speed.clone());
                    MOTOR_COMMANDS_SIGNAL.signal(correction);
                }
            }
        }
    }
}
const P_COEFF: f32 = 10.0;
fn calculate_correction(heading_rad: f32, current_speed: MotorsPower) -> MotorsPower {
    let heading = heading_rad.to_degrees().clamp(0.0, 180.0);
    let error = heading - 90.0;
    let force = error.abs() / 90.0;
    info!("Error {}, h: {}", error,heading);
    if error < 0.0 {
        let l = (20.0 * P_COEFF * force) as i16;
        info!("L increase {}%", l);
        MotorsPower::new(
            increase_by_percent(current_speed.left, l).clamp(0, 255),
            increase_by_percent(current_speed.right, 0),
        )
    } else if error > 0.0 {
        let r = (80.0 * P_COEFF * force) as i16;
        info!("R increase {}%", r);
        MotorsPower::new(
            increase_by_percent(current_speed.left, -30),
            increase_by_percent(current_speed.right, r).clamp(0, 255),
        )
    } else {
        current_speed
    }
}

fn increase_by_percent(initial: i16, by_percents: i16) -> i16 {
    (initial as i32 * (by_percents as i32 + 100) / 100) as i16
}
