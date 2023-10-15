#![no_std]
#![no_main]

mod feather_pwm;
mod usb_serial;
use usb_serial::*;

use ufmt::uwriteln;

use feather_m4::ehal::digital::v2::OutputPin;
use feather_m4::ehal::Pwm;
use feather_m4::hal::clock::GenericClockController;

use feather_m4::hal::delay::Delay;
use feather_m4::hal::prelude::_embedded_hal_blocking_delay_DelayMs;

use panic_halt as _;

//use cortex_m::asm;
//use cortex_m_rt::entry;
use feather_m4::entry;
use feather_m4::hal;
use hal::fugit::RateExtU32;

use cortex_m_rt::exception;

// use hal::gpio::{Pin, Pins, *};
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

    let mut d13 = pins.d13.into_readable_output();
    for _ in 0..10 {
        d13.set_low().unwrap();
        delay.delay_ms(100u32);
        d13.set_high().unwrap();
        delay.delay_ms(100u32);
    }

    let gclk1 = &clocks.gclk1();
    let gclk0 = &clocks.gclk0();

    // Configure the digital pins for PWM
    let tcc1pinout = hal::pwm::TCC1Pinout::Pa16(pins.d5);
    // hal::pwm::TCC1Pinout::Pa18(pins.d6);
    // hal::pwm::TCC1Pinout::Pa19(pins.d9);
    // hal::pwm::TCC1Pinout::Pa20(pins.d10);
    // hal::pwm::TCC1Pinout::Pa21(pins.d11);
    // hal::pwm::TCC1Pinout::Pa22(pins.d12);
    let tcc0pinout = hal::pwm::TCC0Pinout::Pa23(d13);

    //peripherals.TCC1.cc()[7].write(|w| unsafe { w.cc().bits(500) });

    // let mut tcc1pwm = hal::pwm::Tcc1Pwm::new(
    //     &clocks.tcc0_tcc1(gclk0).unwrap(),
    //     1.kHz(),
    //     peripherals.TCC1,
    //     tcc1pinout,
    //     &mut peripherals.MCLK,
    // );

    let mut tcc0pwm = hal::pwm::Tcc0Pwm::new(
        &clocks.tcc0_tcc1(gclk0).unwrap(),
        50.Hz(),
        peripherals.TCC0,
        tcc0pinout,
        &mut peripherals.MCLK,
    );

    let uart = feather_m4::uart(
        &mut clocks,
        125000u32.Hz(), //Hertz::Hz(125000),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        pins.d0,
        pins.d1,
    );

    let i2c = feather_m4::i2c_master(
        &mut clocks,
        125000u32.Hz(), // TODO: figure out frequency
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
    );

    // let (mut rx, _tx) = uart.split();
    // rx.read().unwrap();

    let mut duty = tcc0pwm.get_max_duty();
    //tcc1pwm.enable(hal::pwm::Channel::_0);
    tcc0pwm.enable(hal::pwm::Channel::_3);

    loop {
        uwriteln!(
            UsbSerialWriter,
            "Max Duty: {}\nPeriod: {}\n Current Duty: {}",
            tcc0pwm.get_max_duty(),
            tcc0pwm.get_period().to_Hz(),
            tcc0pwm.get_duty(hal::pwm::Channel::_3)
        )
        .unwrap();

        tcc0pwm.set_duty(hal::pwm::Channel::_3, duty);

        duty -= 1;
        if duty == 0 {
            duty = tcc0pwm.get_max_duty();
            print(b"Looping duty");
        }
        delay.delay_ms(1u32);
    }
}

#[exception]
unsafe fn DefaultHandler(_i: i16) {
    print(b"ASDFADSFDSAFDASF\n");
    loop {}
}
