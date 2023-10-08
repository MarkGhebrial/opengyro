use core::any::Any;

use feather_m4::hal;
use feather_m4::pac;

use hal::gpio::*;
use hal::clock::GenericClockController;

type D5Type = Pin<PA16, AlternateG>;
//type D5Type = AnyPin;
use hal::pwm::TCC1Pinout;
use hal::pwm::Tcc1Pwm;

/// Setup the pwm pins using the the TCC1 peripheral
pub struct FeatherPwm {
    d5: D5Type,
    tc2: pac::TCC1,
}

impl FeatherPwm {
    pub fn init(d5: impl Into<D5Type>, tc2: pac::TCC1, clocks: &mut GenericClockController) -> Self {

        //Tcc1Pwm::
        //let d5 = TCC1Pinout::Pa16(d5);

        let gclk = &clocks.gclk1();
        clocks.tcc0_tcc1(gclk);

        // Set d5's mode
        let d5: Pin<PA16, AlternateG> = d5.into();

        

        FeatherPwm { d5, tc2 }
    }
}
