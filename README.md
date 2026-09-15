# Climber
Car climbs to a hill

## Inspired by
![](media/inspired_by.png)

## Chasis
Arduino Car
![](media/chasis.png)

## Stages
1. Migrates from Arduino/C++ to STM32/Rust/Embassy - controlled from mobile app via BLE
1. Added barometer, accelerometer/gyroscope
1. Replaced mobile app control with controller from notebook (written in Python) for better control and logging/telemetry
1. Migrated to N20 motors
![N20](media/n20.jpeg)
1. Migrated back to TT motors
![TT](media/tt.jpeg)


## Hardware components
```mermaid
---
title: Hardware components
---
flowchart TD
    subgraph Car
        MCU["`MCU
        STM32F446`"]

        Barometer["`Barometer
        BME280`"]
        Barometer --I2C--> MCU

        IMU["`IMU
        MPU6050`"]
        IMU --I2C--> MCU 
        BLE["`BLE
        DX-BT24`"]
        MCU <--UART--> BLE 
        
        motor_driver["Motor Driver
        4WD TB6612"]
        Motors
        motor_driver --PWM--> Motors
        MCU --PWM+GPIO--> motor_driver
    end
    subgraph Control station
        control["Control"]
        Display
    end
    subgraph mobile_app["Mobile app [deprecated]"]
        mobile_app_control["Mobile app control"]
    end
    mobile_app_control --BLE--> BLE
    BLE --BLE--> Display
    control --BLE--> BLE
```

## Firmware components
```mermaid
---
title: Firmware components
---
flowchart TD
    control_command_queue@{ shape: das, label: "control_command_queue" }
    controller_command_reader[controller_command_reader task]
    ble_uart[BLE UART]
    controller_command_reader --> control_command_queue
    ble_uart --> controller_command_reader

    imu_reader[IMU reader task]
    imu_i2c[IMU I2C]
    imu_reader --> control_command_queue
    imu_i2c --> imu_reader

    baro_reader[Barometer reader task]
    baro_i2c[Baro I2C]
    baro_reader --[NOT IMPLEMENTED]--> control_command_queue
    baro_i2c --> baro_reader

    
    subgraph main_controller[Controller task]
        pi_controller["PI controller"]
    end
    control_command_queue --> main_controller

    update_motor[Update motor task]
    motor_driver[Motor driver]
    update_motor --> motor_driver
    motor_commands_signal@{ shape: das, label: "motor_commands_signal" }

    motor_commands_signal --> update_motor
    main_controller --> motor_commands_signal

    heartbeat[Heartbeat task]
```

## Videos
- [controlling](media/controlling.mov)

- climbing

## TODO
- [x] add diagram for data flow and components
- [ ] integral component in PI
- [x] create and move to the new repo
- [x] N20 motors improve handling
- [x] migrate back from N20 motors
- [ ] error handling and reconnects for accelerometer, baro and BLE
- [ ] add led command shows task statuses
- [ ] external configuration
- [ ] beautify Python client, add doc
- [ ] record video
- [x] mise run for client
- [x] watchdog
- [x] test watchdog
- [x] remove watchdog
- [ ] port to SpeedyBee