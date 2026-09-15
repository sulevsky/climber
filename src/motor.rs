use core::{ops::Range, prelude::v1};

use embassy_stm32::{
    gpio::Output,
    timer::{GeneralInstance4Channel, simple_pwm::SimplePwmChannel},
};
use log::info;

use crate::utils::map_in_range;

const MAX_IN_MOTOR_VALUE: u32 = 255;
const MIN_IN_MOTOR_VALUE: i16 = -255;
const ABS_IN_PWM_RANGE: Range<u32> = 0..256;

const CALIBRATED_PWM_VALUE_FORWARD: Range<u32> = 100..190;
const CALIBRATED_PWM_VALUE_BACKWARD: Range<u32> = 100..211;

pub struct Motors<'d, CHL: GeneralInstance4Channel, CHR: GeneralInstance4Channel> {
    left_f_b: Output<'d>,
    left_pwm: SimplePwmChannel<'d, CHL>,
    right_f_b: Output<'d>,
    right_pwm: SimplePwmChannel<'d, CHR>,
}

#[derive(Clone)]
pub struct MotorsPower {
    pub left: i16,
    pub right: i16,
}
impl MotorsPower {
    pub fn new(left: i16, right: i16) -> MotorsPower {
        assert!((MIN_IN_MOTOR_VALUE..=MAX_IN_MOTOR_VALUE as i16).contains(&left));
        assert!((MIN_IN_MOTOR_VALUE..=MAX_IN_MOTOR_VALUE as i16).contains(&right));
        Self { left, right }
    }
    pub fn stop() -> MotorsPower {
        MotorsPower::new(0, 0)
    }
    pub fn reversed(&self) -> MotorsPower {
        MotorsPower::new(-self.left, -self.right)
    }
}

impl<'d, CHL: GeneralInstance4Channel, CHR: GeneralInstance4Channel> Motors<'d, CHL, CHR> {
    pub fn new(
        left_f_b: Output<'d>,
        mut left_pwm: SimplePwmChannel<'d, CHL>,
        right_f_b: Output<'d>,
        mut right_pwm: SimplePwmChannel<'d, CHR>,
    ) -> Self {
        left_pwm.enable();
        right_pwm.enable();

        Self {
            left_f_b,
            left_pwm,
            right_f_b,
            right_pwm,
        }
    }
    pub fn update(&mut self, motors_power: MotorsPower) {
        info!("l/r {}/{}", motors_power.left, motors_power.right);
        if motors_power.left == 0 {
            self.left_f_b.set_low();
            self.left_pwm.set_duty_cycle_fully_off();
        } else if motors_power.left < 0 {
            self.left_f_b.set_low();
            let fraction = map_in_range(
                motors_power.left.abs() as u32,
                ABS_IN_PWM_RANGE,
                CALIBRATED_PWM_VALUE_BACKWARD,
            );
            info!(
                "b l {}/{} init l/r {}/{}",
                fraction, MAX_IN_MOTOR_VALUE, motors_power.left, motors_power.right
            );
            self.left_pwm
                .set_duty_cycle_fraction(fraction, MAX_IN_MOTOR_VALUE);
        } else {
            self.left_f_b.set_high();
            let fraction = map_in_range(
                MAX_IN_MOTOR_VALUE - motors_power.left.abs() as u32,
                ABS_IN_PWM_RANGE,
                CALIBRATED_PWM_VALUE_FORWARD,
            );
            info!(
                "f l {}/{} init l/r {}/{}",
                fraction, MAX_IN_MOTOR_VALUE, motors_power.left, motors_power.right
            );
            self.left_pwm
                .set_duty_cycle_fraction(fraction, MAX_IN_MOTOR_VALUE);
        }
        if motors_power.right == 0 {
            self.right_f_b.set_low();
            self.right_pwm.set_duty_cycle_fully_off();
        } else if motors_power.right < 0 {
            self.right_f_b.set_low();
            let fraction = map_in_range(
                motors_power.right.abs() as u32,
                ABS_IN_PWM_RANGE,
                CALIBRATED_PWM_VALUE_BACKWARD,
            );
            info!(
                "b r {}/{} init l/r {}/{}",
                fraction, MAX_IN_MOTOR_VALUE, motors_power.left, motors_power.right
            );
            self.right_pwm
                .set_duty_cycle_fraction(fraction, MAX_IN_MOTOR_VALUE);
        } else {
            self.right_f_b.set_high();
            let fraction = map_in_range(
                MAX_IN_MOTOR_VALUE - motors_power.right.abs() as u32,
                ABS_IN_PWM_RANGE,
                CALIBRATED_PWM_VALUE_FORWARD,
            );
            info!(
                "f r {}/{} init l/r {}/{}",
                fraction, MAX_IN_MOTOR_VALUE, motors_power.left, motors_power.right
            );
            self.right_pwm
                .set_duty_cycle_fraction(fraction, MAX_IN_MOTOR_VALUE);
        }
    }
}
