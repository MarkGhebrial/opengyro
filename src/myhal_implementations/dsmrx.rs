//use embedded_hal::serial::Read;

use crate::{myhal::reciever::Reciever, timer::UpTimer};

#[allow(non_camel_case_types)]
#[derive(ufmt::derive::uDebug, PartialEq, Clone, Copy)]
pub enum DsmSystem {
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

impl Dsm1024Servo {
    pub fn get_us(&self) -> u16 {
        (self.position as f64 * 1.166) as u16 + 903
    }
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

#[derive(ufmt::derive::uDebug, Clone, Copy)]
pub struct DsmInternalFrame {
    pub fades: u8,
    pub system: DsmSystem,
    pub servos: [Dsm1024Servo; 7],
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
    prev_frame: Option<DsmInternalFrame>,
    has_new_frame: bool,
    timer: UpTimer,
    failsafe_timer: UpTimer,
}

impl DsmRx /*<Rx>*/
// where
//     Rx: Read<u8>,
{
    pub fn new(/*uart: Option<Rx>*/) -> Self {
        DsmRx {
            //rx: uart,
            buffer: [0; 16],
            buffer_index: 0,
            prev_frame: None,
            has_new_frame: false,
            timer: UpTimer::new(),
            failsafe_timer: UpTimer::new(),
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

    /// Handle a byte from the UART.
    ///
    /// You must call this function when a new byte is recieved.
    pub fn handle_serial_event(&mut self, byte: u8) {
        if self.timer.elapsed_ms() > 17 {
            self.clear_buffer();
        }
        // if DsmSystem::from(byte) != DsmSystem::Invalid && self.buffer_index >= 1 {
        //     self.buffer[0] = self.buffer[self.buffer_index - 1];
        //     self.buffer_index = 1;
        // }

        self.buffer[self.buffer_index] = byte;
        self.buffer_index += 1;
        self.timer.reset();

        if self.frame_is_avaliable() {
            self.prev_frame = Some(DsmInternalFrame::from(&self.buffer));
            self.has_new_frame = true;
            self.clear_buffer();

            self.failsafe_timer.reset();
        }
    }

    pub fn frame_is_avaliable(&self) -> bool {
        self.buffer_index >= 16
    }

    pub fn get_frame(&mut self) -> Option<DsmInternalFrame> {
        self.has_new_frame = false;
        self.prev_frame
    }
}

impl Reciever<7> for DsmRx {
    fn has_new_data(&self) -> bool {
        self.has_new_frame
    }

    // TODO: Converting a DsmInternalFrame to an array of ints isn't super fast. Cache the result instead
    // so that we don't waste computing time when a user calls this funtion continuously
    fn get_channels(&mut self) -> [u16; 7] {
        let mut out = [1500; 7]; // We set the default array value as 1500us so that the servos are centered until a frame has been recieved
        out[0] = 0; // We set the first element in the array to 0us since we don't want the throttle to move until a frame has been recieved

        if let Some(frame) = &self.prev_frame {
            for s in &frame.servos {
                out[s.channel_id as usize] = s.get_us();
            }
        }

        self.has_new_frame = false;

        out
    }

    fn is_in_failsafe(&self) -> bool {
        // Spektrum's satellite reciever spec advises that the flight controller should
        // enter failsafe mode after not recieving a frame for longer than one second
        self.failsafe_timer.elapsed_ms() > 1000
    }
}
