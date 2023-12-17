/// Future me is going to forget what all of this is for.
/// Present me needs to document this so that future me can make sense of it

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

use crate::myhal::servos::Servo;

static mut TCC0_PWM: FeatherPwmTimer = FeatherPwmTimer::None;
static mut TCC1_PWM: FeatherPwmTimer = FeatherPwmTimer::None;

enum FeatherPwmTimer {
    None,
    Tcc0(Tcc0Pwm<PA23, AlternateG>),
    Tcc1(Tcc1Pwm<PA16, AlternateF>),
}

pub struct FeatherServo {
    tcc: &'static mut FeatherPwmTimer,
    channel: Channel,
}
impl Servo for FeatherServo {
    fn set_us(&mut self, us: u16) {
        match self.tcc {
            FeatherPwmTimer::Tcc0(timer) => {
                let period_us: f32 = (1.0 / 50.0) * 1000000.0;
                let max_duty = timer.get_max_duty();

                let duty: f32 = us as f32 / period_us; // Get the duty as a percentage (from 0.0 to 1.0)
                let scaled_duty: u32 = (duty * max_duty as f32) as u32;

                timer.set_duty(self.channel, scaled_duty);
            }
            FeatherPwmTimer::Tcc1(timer) => {
                let period_us: f32 = (1.0 / 50.0) * 1000000.0;
                let max_duty = timer.get_max_duty();

                let duty: f32 = us as f32 / period_us; // Get the duty as a percentage (from 0.0 to 1.0)
                let scaled_duty: u32 = (duty * max_duty as f32) as u32;

                timer.set_duty(self.channel, scaled_duty);
            }
            FeatherPwmTimer::None => (),
        }
    }
}

/// Setup the pwm pins using the the TCC1 peripheral
pub struct FeatherPwm {
    pub servo1: FeatherServo, // TODO: should the servo numbering start at 0 instead of 1?
    pub servo2: FeatherServo,
    pub servo3: FeatherServo,
    pub servo4: FeatherServo,
    pub servo5: FeatherServo,
    pub servo6: FeatherServo,
    pub servo7: FeatherServo,
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

        unsafe {
            TCC0_PWM = FeatherPwmTimer::Tcc0(tcc0pwm);
            TCC1_PWM = FeatherPwmTimer::Tcc1(tcc1pwm);

            Self {
                servo1: FeatherServo {
                    tcc: &mut TCC1_PWM,
                    channel: Channel::_0,
                },
                servo2: FeatherServo {
                    tcc: &mut TCC1_PWM,
                    channel: Channel::_2,
                },
                servo3: FeatherServo {
                    tcc: &mut TCC1_PWM,
                    channel: Channel::_3,
                },

                servo4: FeatherServo {
                    tcc: &mut TCC0_PWM,
                    channel: Channel::_0,
                },
                servo5: FeatherServo {
                    tcc: &mut TCC0_PWM,
                    channel: Channel::_1,
                },
                servo6: FeatherServo {
                    tcc: &mut TCC0_PWM,
                    channel: Channel::_2,
                },
                servo7: FeatherServo {
                    tcc: &mut TCC0_PWM,
                    channel: Channel::_3,
                },
            }
        }
    }

    // pub fn set_channel_us(&mut self, channel: u8, us: u16) {
    //     let period_us: f32 = (1.0 / 50.0) * 1000000.0;
    //     let max_duty = self.tcc1pwm.get_max_duty();

    //     let duty: f32 = us as f32 / period_us; // Get the duty as a percentage (from 0.0 to 1.0)
    //     let scaled_duty: u32 = (duty * max_duty as f32) as u32;

    //     match channel {
    //         0 => self.tcc1pwm.set_duty(Channel::_0, scaled_duty),
    //         1 => self.tcc1pwm.set_duty(Channel::_2, scaled_duty),
    //         2 => self.tcc1pwm.set_duty(Channel::_3, scaled_duty),
    //         3 => self.tcc0pwm.set_duty(Channel::_0, scaled_duty),
    //         4 => self.tcc0pwm.set_duty(Channel::_1, scaled_duty),
    //         5 => self.tcc0pwm.set_duty(Channel::_2, scaled_duty),
    //         6 => self.tcc0pwm.set_duty(Channel::_3, scaled_duty),
    //         _ => (),
    //     };
    // }
}
