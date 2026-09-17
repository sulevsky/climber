#![no_std]
#![no_main]

mod baro;
mod demo;
mod health;
mod imu;
mod logger;
mod main_controller;
mod motor;
mod utils;

use bme280_rs::AsyncBme280;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, dma,
    gpio::{Output, OutputType},
    i2c,
    mode::Async,
    peripherals::{self, TIM2, TIM3},
    timer::simple_pwm::{PwmPin, SimplePwm},
    usart::UartRx,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use embassy_time::{Duration, Timer};
use log::{error, info, warn};
use panic_probe as _;
use static_cell::StaticCell;

use crate::{
    baro::{calculate_altitude, calibrated_pressure},
    health::{log_health, update_baro_health, update_imu_health},
    imu::{IMU, Mpu6050IMU},
    main_controller::{Event, MainController},
    motor::{Motors, MotorsPower},
    utils::{parse_bool, parse_u32},
};

bind_interrupts!(
    struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C3_EV => i2c::EventInterruptHandler<peripherals::I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<peripherals::I2C3>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_STREAM2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_STREAM4 => dma::InterruptHandler<peripherals::DMA1_CH4>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
    USART3=> embassy_stm32::usart::InterruptHandler<peripherals::USART3>;
    }
);

pub static MOTOR_COMMANDS_SIGNAL: Signal<CriticalSectionRawMutex, MotorsPower> = Signal::new();
static CONTROL_COMMANDS_QUEUE: Channel<CriticalSectionRawMutex, Event, 32> = Channel::new();

static LEFT_PWM: StaticCell<SimplePwm<'static, TIM3>> = StaticCell::new();
static RIGHT_PWM: StaticCell<SimplePwm<'static, TIM2>> = StaticCell::new();

const PWM_FREQUENCY: embassy_stm32::time::Hertz = embassy_stm32::time::Hertz::hz(40000);
const CALIBRATED_PITCH_DEG: Option<f32> = Some(21.757889);
const IMU_SMA_BUFFER_SIZE: usize = 20;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_print!();
    info!("Starting initialization");
    let p = embassy_stm32::init(Default::default());
    let led = Output::new(
        p.PA5,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );

    info!("Initializing USART3");
    let mut usart3_config = embassy_stm32::usart::Config::default();
    usart3_config.baudrate = 9600;
    let (uart_tx, uart_rx) = embassy_stm32::usart::Uart::new(
        p.USART3,
        p.PC11,
        p.PC10,
        p.DMA1_CH3,
        p.DMA1_CH1,
        Irqs,
        usart3_config,
    )
    .unwrap()
    .split();
    logger::init(Some(uart_tx)).expect("Could not initialize logging");
    info!("Initializing PWM");
    let left_direction = Output::new(
        p.PA10,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );
    let left_pwm = PwmPin::new(p.PA6, OutputType::PushPull);
    let right_pwm = PwmPin::new(p.PB10, OutputType::PushPull);
    let left_pwm = SimplePwm::new(
        p.TIM3,
        Some(left_pwm),
        None,
        None,
        None,
        PWM_FREQUENCY,
        embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
    );
    let left_pwm = LEFT_PWM.init(left_pwm);
    let mut left_pwm = left_pwm.ch1();
    left_pwm.enable();
    let right_direction = Output::new(
        p.PB5,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Low,
    );
    let right_pwm = SimplePwm::new(
        p.TIM2,
        None,
        None,
        Some(right_pwm),
        None,
        PWM_FREQUENCY,
        embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
    );
    let right_pwm = RIGHT_PWM.init(right_pwm);
    let right_pwm = right_pwm.ch3();

    let motor = motor::Motors::new(left_direction, left_pwm, right_direction, right_pwm);

    info!("Initializing I2C baro");
    let i2c_baro = embassy_stm32::i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Irqs,
        embassy_stm32::i2c::Config::default(),
    );

    info!("Initializing I2C IMU");
    let i2c_imu = embassy_stm32::i2c::I2c::new(
        p.I2C3,
        p.PA8,
        p.PC9,
        p.DMA1_CH4,
        p.DMA1_CH2,
        Irqs,
        embassy_stm32::i2c::Config::default(),
    );

    spawner.spawn(imu_reader(i2c_imu).unwrap());
    spawner.spawn(baro_reader(i2c_baro).unwrap());
    spawner.spawn(controller_command_reader(uart_rx).unwrap());
    spawner.spawn(update_motor(motor).unwrap());
    spawner.spawn(main_controller().unwrap());
    spawner.spawn(heartbeat(led).unwrap());
}

#[embassy_executor::task]
async fn controller_command_reader(mut uart_rx: UartRx<'static, Async>) {
    let mut buf = [0u8; 1];
    loop {
        match uart_rx.read(&mut buf).await {
            Ok(()) => {
                info!("[->] {}", buf[0] as char);
                CONTROL_COMMANDS_QUEUE
                    .send(Event::UserCommand(buf[0] as char))
                    .await;
            }
            e => {
                error!("UART error: {:?}", e);
            }
        }
    }
}
#[embassy_executor::task]
async fn imu_reader(
    i2c_imu: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Async,
        embassy_stm32::i2c::Master,
    >,
) {
    let mut imu = Mpu6050IMU::new(i2c_imu).await;
    let pitch = imu.pitch_only().await;
    if pitch.is_err() {
        warn!("Could not setup IMU, IMU is down");
        update_imu_health(false).await;
        return;
    } else {
        update_imu_health(true).await;
    }

    let mut pitch = pitch.unwrap();
    info!(
        "Measured pitch is {} rad, {} deg",
        pitch,
        pitch.to_degrees()
    );
    if CALIBRATED_PITCH_DEG.is_some() {
        pitch = CALIBRATED_PITCH_DEG.unwrap().to_radians();
        info!(
            "Using calibrated pitch instead of measured is {} rads, {} deg",
            pitch,
            pitch.to_degrees()
        );
    }
    let mut imu_sma_buffer = [0f32; IMU_SMA_BUFFER_SIZE];
    loop {
        let mut i = 0;
        while i < imu_sma_buffer.len() {
            let current_heading = imu.heading(pitch).await;
            if let Ok(h) = current_heading {
                update_imu_health(true).await;
                imu_sma_buffer[i] = h;
                i += 1;
            } else {
                info!("Could not fetch heading from IMU");
                update_imu_health(false).await;
            }
            Timer::after_millis(10).await;
        }
        let heading = imu_sma_buffer.iter().sum::<f32>() / imu_sma_buffer.len() as f32;
        info!(
            "Heading (smoothed) is {} rads, {} deg",
            heading,
            heading.to_degrees()
        );
        CONTROL_COMMANDS_QUEUE
            .send(Event::HeadingUpdated(heading))
            .await;
    }
}

#[embassy_executor::task]
async fn baro_reader(
    i2c_baro: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Async,
        embassy_stm32::i2c::Master,
    >,
) {
    let mut bme280 = AsyncBme280::new(i2c_baro, embassy_time::Delay);
    bme280.init().await.unwrap();

    bme280
        .set_sampling_configuration(
            bme280_rs::Configuration::default()
                .with_pressure_oversampling(bme280_rs::Oversampling::Oversample16)
                .with_temperature_oversampling(bme280_rs::Oversampling::Oversample16)
                .with_humidity_oversampling(bme280_rs::Oversampling::Oversample1)
                .with_filter(bme280_rs::Filter::Filter16)
                .with_sensor_mode(bme280_rs::SensorMode::Normal),
        )
        .await
        .unwrap();
    Timer::after_millis(3000).await;
    info!("Calibrating pressure");
    let initial_pressure = calibrated_pressure(&mut bme280).await;
    info!("Calibrated pressure: {} Pa", initial_pressure);
    update_baro_health(true).await;
    loop {
        let current_pressure = calibrated_pressure(&mut bme280).await;
        info!(
            "Altitude is: {} m",
            calculate_altitude(initial_pressure, current_pressure)
        );
    }
}

#[embassy_executor::task]
async fn main_controller() {
    let mut controller = MainController::new();
    loop {
        let controller_command = CONTROL_COMMANDS_QUEUE.receive().await;
        controller.on_event(controller_command);
    }
}

#[embassy_executor::task]
async fn heartbeat(mut led: Output<'static>) {
    loop {
        log_health().await;
        Timer::after_secs(1).await;
        led.set_high();
        Timer::after_secs(1).await;
        led.set_low();
    }
}

#[embassy_executor::task]
async fn update_motor(mut motor: Motors<'static, TIM3, TIM2>) {
    loop {
        let message = MOTOR_COMMANDS_SIGNAL.wait().await;
        motor.update(message);
    }
}
