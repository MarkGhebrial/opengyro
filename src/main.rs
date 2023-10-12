#![no_std]
#![no_main]

mod feather_pwm;

use feather_m4::ehal::digital::v2::OutputPin;
use feather_m4::ehal::Pwm;
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
use hal::fugit::RateExtU32;
use hal::gpio::*;
use hal::prelude;
use hal::pwm;

use heapless::String;

use cortex_m::interrupt::free as disable_interrupts;
use cortex_m_rt::exception;
use feather_m4::pac::interrupt;
use feather_m4::pac::NVIC;

use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

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

    /////////// SETUP USB ///////////
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(feather_m4::usb_allocator(
            pins.usb_dm,
            pins.usb_dp,
            peripherals.USB,
            &mut clocks,
            &mut peripherals.MCLK,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        core_peripherals.NVIC.set_priority(interrupt::USB_OTHER, 1);
        core_peripherals.NVIC.set_priority(interrupt::USB_TRCPT0, 1);
        core_peripherals.NVIC.set_priority(interrupt::USB_TRCPT1, 1);
        NVIC::unmask(interrupt::USB_OTHER);
        NVIC::unmask(interrupt::USB_TRCPT0);
        NVIC::unmask(interrupt::USB_TRCPT1);
    }
    /////////// END SETUP USB ///////////

    // let mut usb_bus = UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(0x239a, 0x0022))
    //     .manufacturer("Adafruit")
    //     .product("Feather M4 Express")
    //     .serial_number("E623C8D35348354652202020FF19102F")
    //     .device_class(USB_CLASS_CDC)
    //     .device_class(usbd_serial::USB_CLASS_CDC)
    //     .build();

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
        1.kHz(),
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
        let max_duty_string: String<10> = String::from(tcc0pwm.get_max_duty());
        let period_string: String<10> = String::from(tcc0pwm.get_period().to_Hz());
        let current_duty_string: String<10> = String::from(tcc0pwm.get_duty(hal::pwm::Channel::_3));

        print(b"\nMax Duty: ");
        print(max_duty_string.as_bytes());
        print(b"\nPeriod: ");
        print(period_string.as_bytes());
        print(b"\nCurrent Duty: ");
        print(current_duty_string.as_bytes());
        tcc0pwm.set_duty(hal::pwm::Channel::_3, duty);

        // This is hella unsafe
        // let p = unsafe { feather_m4::pac::Peripherals::steal() };
        // p.TCC1.cc()[7].write(|w| unsafe { w.bits(duty << 6) });
        // p.TCC1.ctrla.write(|w| w.enable().set_bit());
        // p.TCC1.syncbusy.read();


        // tcc1pwm.set_duty(hal::pwm::Channel::_6, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_5, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_4, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_3, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_2, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_1, duty);
        // tcc1pwm.set_duty(hal::pwm::Channel::_0, duty);
        duty -= 1;
        if duty == 0 { duty = tcc0pwm.get_max_duty(); print(b"Looping duty"); }
        delay.delay_ms(1u32);
    }
}

#[exception]
unsafe fn DefaultHandler(_i: i16) {
    print(b"ASDFADSFDSAFDASF\n");
    loop {};
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

fn print(bytes: &[u8]) {
    disable_interrupts(|_| unsafe {
        if let Some(usb_serial) = USB_SERIAL.as_mut() {
            usb_serial.write(bytes).unwrap();
        }
    });
}

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(serial) = USB_SERIAL.as_mut() {
                usb_dev.poll(&mut [serial]);

                // Make the other side happy
                let mut buf = [0u8; 16];
                let _ = serial.read(&mut buf);
            };
        };
    };
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}
