use feather_m4::hal;
use feather_m4::pac;

use feather_m4::{UsbDm, UsbDp};
use hal::clock::GenericClockController;
use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use cortex_m::interrupt::free as disable_interrupts;
use pac::interrupt;
use pac::NVIC;

pub struct UsbSerialWriter;

impl ufmt::uWrite for UsbSerialWriter {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        print(s.as_bytes());

        Ok(())
    }
}

impl embedded_hal::serial::Write<&[u8]> for UsbSerialWriter {
    type Error = ();

    fn write(&mut self, word: &[u8]) -> nb::Result<(), Self::Error> {
        print(word);

        nb::Result::Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        nb::Result::Ok(())
    }
}

/// Setup the USB peripherial for serial communication
pub fn init_usb_serial(
    dm: impl Into<UsbDm>,
    dp: impl Into<UsbDp>,
    usb_periph: pac::USB,
    clocks: &mut GenericClockController,
    mclk: &mut pac::MCLK,
    nvic: &mut pac::NVIC,
) {
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(feather_m4::usb_allocator(dm, dp, usb_periph, clocks, mclk));
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

        // let mut usb_bus = UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(0x239a, 0x0022))
        //     .manufacturer("Adafruit")
        //     .product("Feather M4 Express")
        //     .serial_number("E623C8D35348354652202020FF19102F")
        //     .device_class(USB_CLASS_CDC)
        //     .device_class(usbd_serial::USB_CLASS_CDC)
        //     .build();
    }

    unsafe {
        nvic.set_priority(interrupt::USB_OTHER, 1);
        nvic.set_priority(interrupt::USB_TRCPT0, 1);
        nvic.set_priority(interrupt::USB_TRCPT1, 1);
        NVIC::unmask(interrupt::USB_OTHER);
        NVIC::unmask(interrupt::USB_TRCPT0);
        NVIC::unmask(interrupt::USB_TRCPT1);
    }
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

pub fn print(bytes: &[u8]) {
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
