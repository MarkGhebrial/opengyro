#![no_std]
#![no_main]

mod feather_pwm;
use cortex_m::peripheral::NVIC;
use feather_m4::hal::clock::v2::types::Pac;
use feather_m4::hal::time::Hertz;
use feather_pwm::*;

mod usb_serial;
use usb_serial::*;

mod dsmrx;
mod timer;
use timer::*;

use ufmt::*;

use hal::clock::GenericClockController;
use hal::dmac::*;
use hal::sercom::dma;
use hal::timer::TimerCounter;

use hal::delay::Delay;
use hal::prelude::*;

use panic_halt as _;

//use cortex_m::asm;
//use cortex_m_rt::entry;
use feather_m4::entry;
use feather_m4::hal;
use hal::fugit::RateExtU32;

use hal::sercom::uart::Flags;

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

    let mut uart = feather_m4::uart(
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

    print(b"Configured I2C\n");

    //let (mut rx, _tx) = uart.split();

    // let mut duty = tcc0pwm.get_max_duty();
    // //tcc1pwm.enable(hal::pwm::Channel::_0);
    // tcc0pwm.enable(hal::pwm::Channel::_3);

    let mut dsm_rx = dsmrx::DsmRx::new();

    loop {
        uwriteln!(UsbSerialWriter, "Main loop: {}", elapsed_ms()).unwrap();

        uwriteln!(UsbSerialWriter, "dma complete: {}", rx_dma.complete()).unwrap();
        if rx_dma.complete() {
            let (chan1, uart, rx_buffer) = rx_dma.wait();
            uwriteln!(UsbSerialWriter, "{:?}", rx_buffer).unwrap();

            rx_dma = uart.receive_with_dma(rx_buffer, chan1, waker);
        }

        delay.delay_ms(5u32);
    }
}
