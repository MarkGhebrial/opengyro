#![no_std]
#![no_main]

pub mod myhal;
use myhal::imu::*;
use myhal::reciever::Reciever;
use myhal::servos::Servo;
use myhal::servos::ServoController;

mod myhal_implementations;
use myhal_implementations::*;

mod feather_pwm;
use feather_pwm::*;

mod usb_serial;
use usb_serial::*;

mod timer;
use timer::*;

use ufmt::*;
use ufmt_float::*;

use feather_m4::hal;
use hal::prelude::*;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::dmac::*;

use icm20948::i2c as icm_i2c;
use icm20948_driver::icm20948::{self, IcmError};

use panic_halt as _;

use feather_m4::entry;

#[entry]
fn main() -> ! {
    //asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let mut peripherals = feather_m4::pac::Peripherals::take().unwrap();
    let mut core_peripherals = feather_m4::pac::CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core_peripherals.SYST, &mut clocks);

    let mut dmac = DmaController::init(peripherals.DMAC, &mut peripherals.PM);
    let channels = dmac.split();
    //let chan0 = channels.0.init(PriorityLevel::LVL0);
    let chan1 = channels.1.init(PriorityLevel::LVL0);

    let pins = feather_m4::Pins::new(peripherals.PORT);

    init_usb_serial(
        pins.usb_dm,
        pins.usb_dp,
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        &mut core_peripherals.NVIC,
    );

    delay.delay_ms(2000u32);

    print(b"Hello\n");

    init_timer(
        peripherals.TC2,
        &mut peripherals.MCLK,
        &mut core_peripherals.NVIC,
        &mut clocks,
    );

    let mode_pin = pins.a2.into_pull_up_input();
    let mut d0 = pins.d0.into_readable_output();
    if mode_pin.is_low().unwrap() {
        print(b"Configuring radio...");
        for _ in 0..4 {
            delay.delay_ms(1u32);
            d0.set_high().unwrap();
            delay.delay_ms(1u32);
            d0.set_low().unwrap();
        }
        delay.delay_ms(1000u32);
        print(b"  ...done\n");
    }

    let mut d13 = pins.d13.into_readable_output();
    for _ in 0..10 {
        d13.set_low().unwrap();
        delay.delay_ms(100u32);
        d13.set_high().unwrap();
        delay.delay_ms(100u32);
    }

    print(b"Done blinking\n");

    let mut pwm = FeatherPwm::init(
        pins.d5,
        pins.d6,
        pins.d9,
        pins.d10,
        pins.d11,
        pins.d12,
        d13,
        peripherals.TCC0,
        peripherals.TCC1,
        &mut peripherals.MCLK,
        &mut clocks,
    );

    let mut servos = ServoController::new([
        pwm.servo1, pwm.servo2, pwm.servo3, pwm.servo4, pwm.servo5, pwm.servo6, pwm.servo7,
    ]);

    print(b"Configured PWM\n");

    let uart = feather_m4::uart(
        &mut clocks,
        115200.Hz(),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        d0,
        pins.d1,
    );

    // Set the bit order to msb first
    // let mut config = uart.disable();
    // config.set_bit_order(hal::sercom::uart::BitOrder::MsbFirst);
    // uart = config.enable();

    const LENGTH: usize = 16;
    let rx_buffer: &'static mut [u8; LENGTH] =
        cortex_m::singleton!(: [u8; LENGTH] = [0x00; LENGTH]).unwrap();

    // // Read bits from the UART until there's a gap of more than 10ms
    // // This ensures that each dma transfer contains a full dsm frame
    let mut timer = UpTimer::new();
    // while timer.elapsed_ms() < 10 {
    //     timer.reset();
    //     nb::block!(uart.read()).unwrap();
    // }
    // for _ in 0..15 {
    //     nb::block!(uart.read()).unwrap();
    // }

    let waker = |_| {};
    let mut rx_dma = uart.receive_with_dma(rx_buffer, chan1, waker);

    print(b"Configured UART\n");

    let i2c = feather_m4::i2c_master(
        &mut clocks,
        125000u32.Hz(), // TODO: figure out frequency
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
    );

    print(b"Configured I2C peripheral\n");

    let imu = icm_i2c::IcmImu::new(i2c, 0x69 /* Nice */);
    let mut imu = match imu {
        Ok(imu) => Some(imu),
        Err(e) => {
            print(b"Error: ");
            match e {
                IcmError::BusError(e) => {
                    print(b"Bus Error: ");

                    match e {
                        hal::sercom::i2c::Error::BusError => print(b"BusError"),
                        hal::sercom::i2c::Error::ArbitrationLost => print(b"ArbitrationLost"),
                        hal::sercom::i2c::Error::LengthError => print(b"LengthError"),
                        hal::sercom::i2c::Error::Nack => print(b"Nack"),
                        hal::sercom::i2c::Error::Timeout => print(b"Timeout"),
                    }
                }
                IcmError::InvalidInput => print(b"Invalid input\n"),
            }
            None
        }
    };
    print(b"Configured IMU object\n");

    if let Some(ref mut imu) = imu {
        imu.set_gyro_sen(icm20948::GyroSensitivity::Sen500dps).ok();
        imu.enable_gyro().ok();
        imu.set_acc_sen(icm20948::AccSensitivity::Sen4g).ok();
        imu.enable_acc().ok();
    }

    let mut imu = myhal::imu::IMU::new(imu.unwrap());

    print(b"Enabled gyro\n");

    let mut dsm_rx = DsmRx::new();

    loop {
        //uwriteln!(UsbSerialWriter, "Main loop: {}", elapsed_ms()).unwrap();

        //uwriteln!(UsbSerialWriter, "dma complete: {}", rx_dma.complete()).unwrap();
        if rx_dma.complete() {
            let (chan1, uart, rx_buffer) = rx_dma.wait();
            //uwriteln!(UsbSerialWriter, "Elapsed: {}ms {:?}", timer.elapsed_ms(), rx_buffer).unwrap();
            timer.reset();

            let bytes = rx_buffer.clone();
            for byte in bytes {
                dsm_rx.handle_serial_event(byte);
            }
            //uwriteln!(UsbSerialWriter, "Buff idx: {}", dsm_rx.buffer_index).unwrap();

            rx_dma = uart.receive_with_dma(rx_buffer, chan1, waker);
        }

        uwrite!(UsbSerialWriter, "Failsafe: {};", dsm_rx.is_in_failsafe()).unwrap();

        uwrite!(UsbSerialWriter, " Channel mins:").ok();
        for ch in 0..7 {
            uwrite!(UsbSerialWriter, " {}", dsm_rx.get_channel_min(ch)).ok();
        }
        print(b"; ");

        uwrite!(UsbSerialWriter, "Setting channels: ").unwrap();
        if dsm_rx.has_new_data() {
            uwrite!(UsbSerialWriter, "{:?} ", dsm_rx.get_channels()).unwrap();

            servos.set_servo_us(dsm_rx.get_channels());
        }
        print(b"\n");

        // Update the filter
        if timer.elapsed_ms() >= 10 {
            //uwriteln!(UsbSerialWriter, "Updating filter: {}ms elapsed ", timer.elapsed_ms()).unwrap();
            imu.update();
            timer.reset();
        }

        let gyro_readings = imu.get_rotations();

        // Print gyro readings
        let x = uFmt_f64::Five(gyro_readings.0 * (180.0 / core::f64::consts::PI));
        let y = uFmt_f64::Five(gyro_readings.1 * (180.0 / core::f64::consts::PI));
        let z = uFmt_f64::Five(gyro_readings.2 * (180.0 / core::f64::consts::PI));

        uwrite!(UsbSerialWriter, "Gyro x: {}, y: {}, z: {} ", x, y, z).unwrap();

        //delay.delay_ms(5u32);
    }
}
