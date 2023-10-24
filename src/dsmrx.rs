use embedded_hal::serial::Read;

use crate::timer::UpTimer;
use crate::usb_serial::print;

#[allow(non_camel_case_types)]
#[derive(ufmt::derive::uDebug, PartialEq)]
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

#[derive(ufmt::derive::uDebug, Clone, Copy)]
pub struct Dsm1024Servo {
    pub channel_id: u8,
    pub position: u16,
}

impl From<u16> for Dsm1024Servo {
    fn from(value: u16) -> Self {
        Self {
            channel_id: ((value & 0xFC00) >> 10) as u8,
            position: value & 0x03FF,
        }
    }
}

impl From<[u8; 2]> for Dsm1024Servo {
    fn from(value: [u8; 2]) -> Self {
        Self::from(u16::from_be_bytes(value))
    }
}

#[derive(ufmt::derive::uDebug)]
pub struct DsmInternalFrame {
    fades: u8,
    system: DsmSystem,
    servos: [Dsm1024Servo; 7],
}

impl From<&[u8; 16]> for DsmInternalFrame {
    fn from(bytes: &[u8; 16]) -> Self {
        let mut servos: [Dsm1024Servo; 7] = [Dsm1024Servo {
            channel_id: 0,
            position: 0,
        }; 7];

        for i in 1..8 {
            let index = i * 2 as usize;
            let servo: [u8; 2] = [bytes[index], bytes[index + 1]];

            servos[i - 1] = Dsm1024Servo::from(servo);
        }

        Self {
            fades: bytes[0],
            system: bytes[1].into(),
            servos: servos,
        }
    }
}

pub struct DsmRx /*<Rx: Read<u8>>*/ {
    //rx: Option<Rx>,
    pub buffer: [u8; 16],
    pub buffer_index: usize,
    timer: UpTimer,
}

impl DsmRx /*<Rx>*/
// where
//     Rx: Read<u8>,
{
    pub fn new(/*uart: Option<Rx>*/) -> Self {
        let timer = UpTimer::new();

        DsmRx {
            //rx: uart,
            buffer: [0; 16],
            buffer_index: 0,
            timer,
        }
    }

    // pub fn poll(&mut self) {
    //     match self.rx.read() {
    //         Err(nb::Error::WouldBlock) => return,
    //         Err(_e) => {
    //             print(b"UART Error");
    //             return;
    //         }
    //         Ok(byte) => self.handle_serial_event(byte),
    //     }
    // }

    fn clear_buffer(&mut self) {
        self.buffer_index = 0;
        self.buffer = [0; 16];
    }

    pub fn handle_serial_event(&mut self, byte: u8) {
        // if self.timer.elapsed_ms() > 17 {
        //     self.clear_buffer();
        // }
        if DsmSystem::from(byte) != DsmSystem::Invalid && self.buffer_index >= 1 {
            self.buffer[0] = self.buffer[self.buffer_index - 1];
            self.buffer_index = 1;
        }

        self.buffer[self.buffer_index] = byte;
        self.buffer_index += 1;
        self.timer.reset();
    }

    pub fn frame_is_avaliable(&self) -> bool {
        self.buffer_index >= 16
    }

    pub fn parse_frame(&mut self) -> DsmInternalFrame {
        let frame = DsmInternalFrame::from(&self.buffer);
        self.clear_buffer();
        frame
    }
}
