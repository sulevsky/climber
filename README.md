# Climber
Car climbs to a hill

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
    A@{ shape: das, label: "CONTROL_COMMANDS_QUEUE" }
    controller_command_reader[controller_command_reader task]
    ble_uart[BLE UART]
    controller_command_reader --> A
    ble_uart --> controller_command_reader

    imu_reader[IMU reader task]
    imu_i2c[IMU I2C]
    imu_reader --> A
    imu_i2c --> imu_reader

    baro_reader[Barometer reader task]
    baro_i2c[Baro I2C]
    baro_reader --[NOT IMPLEMENTED]--> A
    baro_i2c --> baro_reader

    main_controller[Controller task]
    A --> main_controller

    update_motor[Update motor task]
    motor_driver[Motor driver]
    update_motor --> motor_driver
    B@{ shape: das, label: "MOTOR_COMMANDS_QUEUE" }

    B --> update_motor
    main_controller --> B

    heartbeat[Heartbeat task]
```
## TODO
- [x] add diagram for data flow and components
- [ ] integral component in PID
- [x] create and move to the new repo
- [x] N20 motors improve handling
- [ ] migrate back from N20 motors
- [ ] port to SpeedyBee
- [ ] error handling and reconnects for accelerometer, baro and BLE
- [ ] add led command shows task statuses
- [ ] external configuration
- [ ] beautify Python client, add doc
- [ ] mise run for client
- [ ] record video
- [x] watchdog
- [x] test watchdog
- [x] remove watchdog