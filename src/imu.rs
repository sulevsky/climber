use embassy_time::{Duration, with_timeout};
use log::{error, info};
use mpu6050_async::Mpu6050;

use micromath::F32Ext;

const RECONNECTS_NUM: u8 = 3;

pub struct Mpu6050IMU<I2C: embedded_hal_async::i2c::I2c> {
    mpu: Mpu6050<I2C>,
    is_up: bool,
    delay: embassy_time::Delay,
}

impl<I2C: embedded_hal_async::i2c::I2c> Mpu6050IMU<I2C> {
    pub async fn try_connect(&mut self) {
        for i in 1..=RECONNECTS_NUM {
            info!("Connecting to IMU, try: {}", i);
            let connect_result =
                with_timeout(Duration::from_secs(2), self.mpu.init(&mut self.delay)).await;
            match connect_result {
                Ok(Ok(())) => {
                    info!("Connected to IMU");
                    self.is_up = true;
                    return;
                }
                Ok(Err(e)) => {
                    info!("Could not connect to IMU: {:?}", e);
                }
                Err(e) => {
                    info!("Could not connect to IMU because of timeout: {:?}", e);
                }
            }
        }
    }
    pub async fn new(i2c: I2C) -> Self {
        let mut r = Self {
            mpu: mpu6050_async::Mpu6050::new(i2c),
            is_up: false,
            delay: embassy_time::Delay,
        };
        r.try_connect().await;
        r
    }
}

pub trait IMU {
    async fn pitch_only(&mut self) -> Result<f32, ()>;
    async fn heading(&mut self, pitch: f32) -> Result<f32, ()>;
    async fn get_acc_retrying(&mut self) -> Result<(f32, f32, f32), ()>;
}

impl<I2C: embedded_hal_async::i2c::I2c> IMU for Mpu6050IMU<I2C> {
    /**
     * Rads
     */
    async fn pitch_only(&mut self) -> Result<f32, ()> {
        if !self.is_up {
            info!("Not connected to IMU, reconnecting");
            self.try_connect().await;
        }
        if self.is_up {
            let angles = self.mpu.get_acc_angles().await;
            match angles {
                Ok(a) => {
                    return Ok(a.y);
                }
                Err(e) => {
                    error!("Error fetching pitch from IMU, {:?}", e);
                    self.is_up = false;
                    return Err(());
                }
            }
        } else {
            error!("Could not connect to IMU");
            Err(())
        }
    }

    /**
     * heading in radians
     * 0-left, Pi/2 - forward, Pi - right
     * consider rework to 2 or 3 vectors
     * required: pitch > 0
     */
    async fn heading(&mut self, pitch_rads: f32) -> Result<f32, ()> {
        if let Ok((_, ay, _)) = self.get_acc_retrying().await {
            let sin_alpha = pitch_rads.sin();
            let ratio = ay / sin_alpha;
            let ratio = ratio.clamp(-1.0, 1.0);
            let heading_magnitude = ratio.acos();
            Ok(heading_magnitude)
        } else {
            error!("Could not fetch heading from IMU");
            Err(())
        }
    }

    async fn get_acc_retrying(&mut self) -> Result<(f32, f32, f32), ()> {
        if self.is_up {
            let accel = with_timeout(Duration::from_millis(500), self.mpu.get_acc()).await;
            match accel {
                Ok(Ok(a)) => {
                    return Ok((a.x, a.y, a.z));
                }
                Err(e) => {
                    error!("Could not connect to IMU, retrying, {:?}", e);
                    self.is_up = false;
                }
                Ok(Err(e)) => {
                    error!("Could not connect to IMU, retrying, {:?}", e);
                    self.is_up = false;
                }
            };
        }

        self.try_connect().await;
        if self.is_up {
            let accel = with_timeout(Duration::from_millis(500), self.mpu.get_acc()).await;
            match accel {
                Ok(Ok(a)) => {
                    return Ok((a.x, a.y, a.z));
                }
                Err(e) => {
                    error!("Could not reconnect to IMU, {:?}", e);
                }
                Ok(Err(e)) => {
                    error!("Could not reconnect to IMU, {:?}", e);
                }
            }
        }

        error!("Could not reconnect to IMU");
        self.is_up = false;
        Err(())
    }
}
