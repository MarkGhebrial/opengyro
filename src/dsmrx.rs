use embedded_hal::serial::Read;

use crate::usb_serial::print;

#[allow(non_camel_case_types)]
enum DsmSystem {
    DsmX_11Ms = 0x01,
    DsmX_22Ms = 0x12,
    Dsm2_11Ms = 0xa2,
    Dsm2_22Ms = 0xb2,
    Invalid,
}

impl From<u8> for DsmSystem {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Dsm2_22Ms,
            0x12 => Self::Dsm2_11Ms,
            0xa2 => Self::DsmX_22Ms,
            0xb2 => Self::DsmX_11Ms,
            _ => Self::Invalid,
        }
    }
}

struct DsmInternal {
    fades: u8,
    system: u8,
}

pub struct DsmRx<Rx: Read<u8>> {
    rx: Rx,
    buffer: [u8; 20],
    buffer_index: usize,
}

impl<Rx> DsmRx<Rx>
where
    Rx: Read<u8>,
{
    pub fn new(uart: Rx) -> Self {
        DsmRx {
            rx: uart,
            buffer: [0; 20],
            buffer_index: 0,
        }
    }

    pub fn poll(&mut self) {
        match self.rx.read() {
            Err(nb::Error::WouldBlock) => return,
            Err(_e) => {
                print(b"UART Error");
                return;
            }
            Ok(byte) => self.handle_serial_event(byte),
        }
    }

    pub fn handle_serial_event(&mut self, byte: u8) {
        self.buffer[self.buffer_index] = byte;
        self.buffer_index += 1;
    }
}
