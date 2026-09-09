use crate::motor::Motors;
use crate::motor::MotorsPower;
use log::info;
use embassy_stm32::timer::GeneralInstance4Channel;

async fn test_motor<'d, CHL: GeneralInstance4Channel, CHR: GeneralInstance4Channel>(
    motor: &mut Motors<'d, CHL, CHR>,
) {
    info!("zero stop");
    motor.update(MotorsPower::new(0, 0));
    embassy_time::Timer::after_secs(3).await;

    info!("left f");
    motor.update(MotorsPower::new(100, 0));
    embassy_time::Timer::after_secs(3).await;
    motor.update(MotorsPower::new(0, 0));
    info!("left b");
    motor.update(MotorsPower::new(-100, 0));
    embassy_time::Timer::after_secs(3).await;
    motor.update(MotorsPower::new(0, 0));
    info!("right f");
    motor.update(MotorsPower::new(0, 100));
    embassy_time::Timer::after_secs(5).await;
    motor.update(MotorsPower::new(0, 0));
    info!("right b");
    motor.update(MotorsPower::new(0, -100));
    embassy_time::Timer::after_secs(5).await;
    motor.update(MotorsPower::new(0, 0));

    for i in (-255..=255).step_by(20) {
        info!("l{}", i);
        motor.update(MotorsPower::new(i, 0));
        embassy_time::Timer::after_millis(500).await;
    }
    for i in (-255..=255).step_by(20) {
        info!("r{}", i);
        motor.update(MotorsPower::new(0, i));
        embassy_time::Timer::after_millis(500).await;
    }
    info!("both f");
    motor.update(MotorsPower::new(255, 255));
    embassy_time::Timer::after_secs(3).await;
    info!("both b");
    motor.update(MotorsPower::new(-255, -255));
    embassy_time::Timer::after_secs(5).await;
    info!("DONE");
}

async fn fblr<'d, CHL: GeneralInstance4Channel, CHR: GeneralInstance4Channel>(
    motor: &mut Motors<'d, CHL, CHR>,
) {
    info!("zero stop");
    motor.update(MotorsPower::new(0, 0));
    embassy_time::Timer::after_secs(3).await;

    info!("r");
    motor.update(MotorsPower::new(100, -100));
    embassy_time::Timer::after_secs(3).await;
    motor.update(MotorsPower::new(0, 0));
    info!("l");
    motor.update(MotorsPower::new(-100, 100));
    embassy_time::Timer::after_secs(3).await;
    motor.update(MotorsPower::new(0, 0));

    info!("both f");
    motor.update(MotorsPower::new(255, 255));
    embassy_time::Timer::after_secs(3).await;
    info!("both b");
    motor.update(MotorsPower::new(-255, -255));
    embassy_time::Timer::after_secs(5).await;
    info!("DONE");
}
