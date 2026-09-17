use log::{error, info};

use crate::{MOTOR_COMMANDS_SIGNAL, motor::MotorsPower};

pub const NORMAL_SPEED: i16 = 85;

pub enum Event {
    UserCommand(char),
    HeadingUpdated(f32),
}

pub struct MainController {
    is_heading_alignment_mode: bool,
    current_speed: MotorsPower,
    pi_controller: PIController,
}

impl MainController {
    pub fn new() -> Self {
        Self {
            is_heading_alignment_mode: false,
            current_speed: MotorsPower::stop(),
            pi_controller: PIController::new(),
        }
    }
    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::UserCommand(controller_command) => {
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
                        -self.current_speed.left,
                        self.current_speed.right,
                    )),
                    //right
                    'd' => Some(MotorsPower::new(
                        self.current_speed.left,
                        -self.current_speed.right,
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
                    let correction = self
                        .pi_controller
                        .calculate_correction(heading, self.current_speed.clone());
                    MOTOR_COMMANDS_SIGNAL.signal(correction);
                }
            }
        }
    }
}
struct PIController {
    cummulative_error: f32,
}

impl PIController {
    const P_COEFF: f32 = 200.0;
    const I_COEFF: f32 = 5.0;
    fn new() -> Self {
        Self {
            cummulative_error: 0.0,
        }
    }
    fn calculate_correction(
        &mut self,
        heading_rad: f32,
        current_speed: MotorsPower,
    ) -> MotorsPower {
        let heading = heading_rad.to_degrees().clamp(0.0, 180.0);
        let error = heading - 90.0;
        let relative_error = error / 90.0;
        self.cummulative_error += relative_error;
        let p_component = Self::P_COEFF * relative_error;
        let i_component = Self::I_COEFF * self.cummulative_error;
        let correction = p_component + i_component;
        info!(
            "error: {} deg, heading: {} deg, P {}, I {}, sum {}",
            error, heading, p_component, i_component, correction
        );
        if correction < -1.0 {
            info!("L increase {}%", correction);
            MotorsPower::new(
                increase_by_percent(current_speed.left, correction.abs() as i16).clamp(0, 255),
                current_speed.right,
            )
        } else if correction > 1.0 {
            info!("R increase {}%", correction);
            MotorsPower::new(
                current_speed.left,
                increase_by_percent(current_speed.right, correction as i16).clamp(0, 255),
            )
        } else {
            current_speed
        }
    }
}
fn increase_by_percent(initial: i16, by_percents: i16) -> i16 {
    (initial as i32 * (by_percents as i32 + 100) / 100) as i16
}
