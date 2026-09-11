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

## Future improvements and todos
- [ ] add diagram for data flow and components
- [ ] integral component in PID
- [x] create and move to the new repo
- [ ] N20 motors improve handling
- [ ] port to SpeedyBee
- [ ] error handling and reconnects for accelerometer, baro and BLE
- [ ] add led command shows task statuses
- [ ] external configuration
- [ ] beautify Python client, add doc
- [ ] record video
- [x] watchdog
- [ ] test watchdog