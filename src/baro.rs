use bme280_rs::AsyncBme280;
use embassy_time::{Delay, Timer};
use log::{error, info};
use micromath::F32Ext;

pub fn calculate_altitude(base_pressure: f32, current_pressure: f32) -> f32 {
    44330.0 * (1.0 - (current_pressure / base_pressure).powf(0.1903))
}

pub async fn calibrated_pressure(
    bme280: &mut AsyncBme280<
        embassy_stm32::i2c::I2c<'_, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
        Delay,
    >,
) -> f32 {
    let mut buf = [0f32; 100];
    for i in 0..100 {
        if let Ok(Some(pressure)) = bme280.read_pressure().await {
            buf[i] = pressure;
        } else {
            error!("Failed pressure reading");
        }
        Timer::after_millis(10).await;
    }
    buf.iter().sum::<f32>() / 100.0
}
