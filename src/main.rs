#![no_std]
#![no_main]

mod feather_pwm;
use feather_m4::hal::clock::v2::dfll::OpenLoop;
use feather_pwm::*;

mod usb_serial;
use usb_serial::*;

mod dsmrx;
use dsmrx::*;

mod timer;
use timer::*;

use ufmt::*;

use feather_m4::hal;
use hal::prelude::*;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::dmac::*;

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

    print(b"Configured I2C\n");

    let mut dsm_rx = DsmRx::new();
    let mut latest_frame: Option<DsmInternalFrame> = None;

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
                if dsm_rx.frame_is_avaliable() {
                    let frame = dsm_rx.parse_frame();
                    latest_frame = Some(frame);
                    //uwriteln!(UsbSerialWriter, "{:?}", frame).unwrap();
                }
            }
            //uwriteln!(UsbSerialWriter, "Buff idx: {}", dsm_rx.buffer_index).unwrap();

            rx_dma = uart.receive_with_dma(rx_buffer, chan1, waker);
        }

        uwrite!(UsbSerialWriter, "Setting channels: ").unwrap();
        if let Some(ref frame) = latest_frame {
            for servo in frame.servos {
                uwrite!(UsbSerialWriter, "{} -> {}us; ", servo.channel_id, servo.get_us()).unwrap();
                pwm.set_channel_us(servo.channel_id, servo.get_us());
            }
        }
        print(b"\n");

        //delay.delay_ms(5u32);
    }
}
