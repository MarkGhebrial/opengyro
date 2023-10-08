#![no_std]
#![no_main]

mod feather_pwm;

use feather_m4::ehal::blocking::i2c::*;
use feather_m4::ehal::digital::v2::OutputPin;
use feather_m4::hal::clock::GenericClockController;

use feather_m4::hal::delay::Delay;
use feather_m4::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

//use cortex_m::asm;
//use cortex_m_rt::entry;
use feather_m4::entry;
use feather_m4::hal;
use hal::gpio::*;
use hal::pwm;
//use hal::prelude;
use hal::fugit::*;

// use hal::gpio::{Pin, Pins, *};
// use hal::sercom::{uart, IoSet1};

#[entry]
fn main() -> ! {
    //asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let mut peripherals = feather_m4::pac::Peripherals::take().unwrap();
    let core_peripherals = feather_m4::pac::CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = Delay::new(core_peripherals.SYST, &mut clocks);

    let pins = feather_m4::Pins::new(peripherals.PORT);

    //let d5: hal::gpio::Pin<PA16, AlternateE> = pins.d5.into_mode();
    //feather_pwm::FeatherPwm::init(pins.d5, peripherals.TCC1, &mut clocks);

    //peripherals.TCC1.
    peripherals
        .TCC1
        .per()
        .write(|w| unsafe { w.per().bits(u32::MAX) });

    peripherals.TCC1.cc()[0].write(|w| unsafe { w.cc().bits(0x1111 << 5) });

    let glck = &clocks.gclk1();
    clocks.tc2_tc3(glck).unwrap();

    // feather_m4::hal::pwm::Pwm1::new(
    //     &clocks.tc0_tc1(&clocks.gclk0()).unwrap(),
    //     1u32.Hz(), //TODO: Figure this out
    //     peripherals.TC1,
    //     pinout,
    //     &mut peripherals.MCLK
    // );

    // let d5 = pins.d5.into_readable_output();

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

    let mut led_pin = pins.d13.into_push_pull_output();

    // let pm = peripherals.MCLK;
    // let sercom = peripherals.SERCOM5;

    // let pads = uart::Pads::<Sercom5, IoSet1>::default()
    //     .rx(pins.d0)
    //     .tx(pins.d1);

    //let config = uart::Config::new(&pm, sercom, pads, 10.m);

    loop {
        // your code goes here
        led_pin.set_high().unwrap();
        delay.delay_ms(500u32);
        led_pin.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
