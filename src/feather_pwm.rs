use embedded_hal::Pwm;
use feather_m4::hal;
use feather_m4::pac;

use hal::clock::GenericClockController;
use hal::gpio::*;
use hal::pwm::*;

use hal::pwm::Channel;
use hal::pwm::TCC1Pinout;
use hal::pwm::Tcc1Pwm;

use fugit::RateExtU32;

/// Setup the pwm pins using the the TCC1 peripheral
pub struct FeatherPwm {
    tcc0pwm: Tcc0Pwm<PA23, AlternateG>,
    tcc1pwm: Tcc1Pwm<PA16, AlternateF>,
}

impl FeatherPwm {
    pub fn init(
        d5: impl AnyPin<Id = PA16>,
        d6: impl AnyPin<Id = PA18>,
        d9: impl AnyPin<Id = PA19>,
        d10: impl AnyPin<Id = PA20>,
        d11: impl AnyPin<Id = PA21>,
        d12: impl AnyPin<Id = PA22>,
        d13: impl AnyPin<Id = PA23>,
        tcc0: pac::TCC0,
        tcc1: pac::TCC1,
        mclk: &mut pac::MCLK,
        clocks: &mut GenericClockController,
    ) -> Self {
        //let gclk1 = &clocks.gclk1();

        // Configure the digital pins for PWM
        let tcc1pinout = TCC1Pinout::Pa16(d5);
        TCC1Pinout::Pa18(d6);
        TCC1Pinout::Pa19(d9);
        TCC0Pinout::Pa20(d10);
        TCC0Pinout::Pa21(d11);
        TCC0Pinout::Pa22(d12);
        let tcc0pinout = TCC0Pinout::Pa23(d13);

        let gclk0 = &clocks.gclk0();
        let clock = clocks.tcc0_tcc1(gclk0).unwrap();

        let tcc1pwm = hal::pwm::Tcc1Pwm::new(&clock, 50.Hz(), tcc1, tcc1pinout, mclk);

        let tcc0pwm = hal::pwm::Tcc0Pwm::new(&clock, 50.Hz(), tcc0, tcc0pinout, mclk);

        FeatherPwm { tcc0pwm, tcc1pwm }
    }

    pub fn set_channel_us(&mut self, channel: u8, us: u16) {
        let period_us: f32 = (1.0 / 50.0) * 1000000.0;
        let max_duty = self.tcc1pwm.get_max_duty();

        let duty: f32 = us as f32 / period_us; // Get the duty as a percentage (from 0.0 to 1.0)
        let scaled_duty: u32 = (duty * max_duty as f32) as u32;

        match channel {
            0 => self.tcc1pwm.set_duty(Channel::_0, scaled_duty),
            1 => self.tcc1pwm.set_duty(Channel::_2, scaled_duty),
            2 => self.tcc1pwm.set_duty(Channel::_3, scaled_duty),
            3 => self.tcc0pwm.set_duty(Channel::_0, scaled_duty),
            4 => self.tcc0pwm.set_duty(Channel::_1, scaled_duty),
            5 => self.tcc0pwm.set_duty(Channel::_2, scaled_duty),
            6 => self.tcc0pwm.set_duty(Channel::_3, scaled_duty),
            _ => (),
        };
    }
}
