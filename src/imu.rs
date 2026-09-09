use embassy_stm32::mode::Async;
use embassy_time::{Duration, with_timeout};
use embedded_hal_async::delay::DelayNs;
use log::info;
use mpu6050_async::{Mpu6050, PI};

use micromath::F32Ext;

const G: f32 = 9.80665;
pub struct Mpu6050IMU<I2C: embedded_hal_async::i2c::I2c> {
    mpu: Mpu6050<I2C>,
    is_up: bool,
    delay: embassy_time::Delay,
}

impl<I2C: embedded_hal_async::i2c::I2c> Mpu6050IMU<I2C> {
    pub async fn try_connect(mpu: &mut Mpu6050<I2C>, delay: &mut embassy_time::Delay) -> bool {
        for i in 1..4 {
            info!("Connecting to IMU, try: {}", i);
            let connect_result = with_timeout(Duration::from_secs(2), mpu.init(delay)).await;
            match connect_result {
                Ok(Ok(_)) => {
                    info!("Connected to IMU");
                    return true;
                }
                Ok(Err(_)) | Err(_) => {
                    info!("Could not connect to IMU");
                }
            }
        }
        false
    }
    pub async fn new(i2c: I2C) -> Self {
        let mut mpu = mpu6050_async::Mpu6050::new(i2c);
        let mut delay = embassy_time::Delay;

        let is_up = Self::try_connect(&mut mpu, &mut delay).await;
        Self { mpu, is_up, delay }
    }
}

pub trait IMU {
    async fn pitch_only(&mut self) -> f32;
    async fn heading(&mut self, pitch: f32) -> f32;

    async fn get_acc_retrying(&mut self) -> (f32, f32, f32);
}

impl<I2C: embedded_hal_async::i2c::I2c> IMU for Mpu6050IMU<I2C> {
    async fn pitch_only(&mut self) -> f32 {
        if self.is_up {
            let angles = self.mpu.get_acc_angles().await;
            match angles {
                Ok(a) => {
                    return a.y;
                }
                Err(_) => {
                    info!("Could not connect to IMU, retrying");
                }
            };
        }
        self.is_up = Self::try_connect(&mut self.mpu, &mut self.delay).await;
        if self.is_up {
            let angles = self.mpu.get_acc_angles().await;
            match angles {
                Ok(a) => {
                    return a.y;
                }
                Err(_) => {
                    info!("Could not connect to IMU, rerunerd default");
                    return 0.0;
                }
            }
        } else {
            info!("Could not connect to IMU");
            0.0
        }
    }

    /**
     * rads 0-left, 90 - forward, 180 - right
     * TODO vova rework to 2 or 3 vectors
     */
    async fn heading(&mut self, pitch: f32) -> f32 {
        //    pitch > 0
        let (ax, ay, az) = self.get_acc_retrying().await;

        // info!(
        //     "ax{} ay{} az{}",
        //     (ax * 100.0) as i32,
        //     (ay * 100.0) as i32,
        //     (az * 100.0) as i32
        // );
        let sin_alpha = pitch.sin();
        let ratio = ay / sin_alpha;
        let ratio = ratio.clamp(-1.0, 1.0);
        let heading_magnitude = ratio.acos();
        // if ay < 0.0 {
        //     heading_magnitude
        // } else {
        //     -heading_magnitude
        // }
        heading_magnitude
    }

    async fn get_acc_retrying(&mut self) -> (f32, f32, f32) {
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;
        if self.is_up {
            let accel = self.mpu.get_acc().await;
            match accel {
                Ok(a) => {
                    ax = a.x;
                    ay = a.y;
                    az = a.z;
                }
                Err(_) => {
                    info!("Could not connect to IMU, retrying");
                    self.is_up = false;
                }
            };
        }
        if !self.is_up {
            self.is_up = Self::try_connect(&mut self.mpu, &mut self.delay).await;
            if self.is_up {
                let accel = self.mpu.get_acc().await;
                match accel {
                    Ok(a) => {
                        ax = a.x;
                        ay = a.y;
                        az = a.z;
                    }
                    Err(_) => {
                        info!("Could not connect to IMU, rerunerd default");
                    }
                }
            } else {
                info!("Could not connect to IMU");
                self.is_up = false;
            }
        }

        (ax, ay, az)
    }
}
