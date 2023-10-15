#![no_std]
#![no_main]

mod feather_pwm;
use embedded_hal::digital::v2::InputPin;
use feather_pwm::*;
mod usb_serial;
use usb_serial::*;

use ufmt::uwriteln;

use feather_m4::ehal::digital::v2::OutputPin;
use feather_m4::ehal::Pwm;
use feather_m4::hal::clock::GenericClockController;

use feather_m4::hal::delay::Delay;
use feather_m4::hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use nb;
use embedded_hal::serial::Read;

use panic_halt as _;

//use cortex_m::asm;
//use cortex_m_rt::entry;
use feather_m4::entry;
use feather_m4::hal;
use hal::fugit::RateExtU32;

use cortex_m_rt::exception;

// use hal::sercom::{uart, IoSet1};

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

    let pins = feather_m4::Pins::new(peripherals.PORT);

    init_usb_serial(
        pins.usb_dm,
        pins.usb_dp,
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        &mut core_peripherals.NVIC,
    );

    print(b"Hello\n");

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

    let pwm = FeatherPwm::init(
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

    print(b"Configured PWM\n");

    let uart = feather_m4::uart(
        &mut clocks,
        115200.Hz(), //Hertz::Hz(125000),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        d0,
        pins.d1,
    );

    print(b"Configured UART\n");

    let i2c = feather_m4::i2c_master(
        &mut clocks,
        125000u32.Hz(), // TODO: figure out frequency
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
    );

    print(b"Configured I2C\n");

    let (mut rx, _tx) = uart.split();

    // let mut duty = tcc0pwm.get_max_duty();
    // //tcc1pwm.enable(hal::pwm::Channel::_0);
    // tcc0pwm.enable(hal::pwm::Channel::_3);

    loop {
        print(b"Reading from UART...\n");
        let frame: u8 = rx.read().unwrap();
        print(&[frame]);

        //uwriteln!(UsbSerialWriter, "Mode pin: {}", mode_pin.is_high().unwrap()).unwrap();

        // uwriteln!(
        //     UsbSerialWriter,
        //     "Max Duty: {}\nPeriod: {}\n Current Duty: {}",
        //     tcc0pwm.get_max_duty(),
        //     tcc0pwm.get_period().to_Hz(),
        //     tcc0pwm.get_duty(hal::pwm::Channel::_3)
        // )
        // .unwrap();

        // tcc0pwm.set_duty(hal::pwm::Channel::_3, duty);

        // duty -= 1;
        // if duty == 0 {
        //     duty = tcc0pwm.get_max_duty();
        //     print(b"Looping duty");
        // }
        delay.delay_ms(5u32);
    }
}

#[exception]
unsafe fn DefaultHandler(_i: i16) {
    print(b"ASDFADSFDSAFDASF\n");
    loop {}
}
