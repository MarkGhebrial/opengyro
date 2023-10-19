use feather_m4::hal;
use feather_m4::pac;

use hal::prelude::*;
use fugit::HertzU32 as Hertz;

use hal::clock::GenericClockController;
use hal::timer::TimerCounter;

use pac::interrupt;
use pac::NVIC;

pub fn init_timer(
    tc2: pac::TC2,
    mclk: &mut pac::MCLK,
    nvic: &mut pac::NVIC,
    clocks: &mut GenericClockController,
) {
    let gclk1 = &clocks.gclk1();
    let tcclk = clocks.tc2_tc3(gclk1).unwrap();
    let mut timer = TimerCounter::tc2_(&tcclk, tc2, mclk);

    unsafe {
        nvic.set_priority(interrupt::TC2, 1);
        NVIC::unmask(interrupt::TC2);
    }

    timer.start(Hertz::kHz(1).into_duration());
    timer.enable_interrupt();
}

static mut ELAPSED_MS: u64 = 0;

pub fn elapsed_ms() -> u64 {
    unsafe { ELAPSED_MS }
}

#[interrupt]
fn TC2() {
    unsafe {
        // TODO: Don't steal the peripherials!!
        let tc2 = feather_m4::pac::Peripherals::steal().TC2;
        tc2.count16().intflag.write(|w| w.ovf().set_bit());

        ELAPSED_MS += 1;
    }
}


pub struct UpTimer {
    initial_time: u64,
}

impl UpTimer {
    pub fn new() -> Self {
        Self {
            initial_time: elapsed_ms()
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        elapsed_ms() - self.initial_time
    }

    pub fn reset(&mut self) {
        self.initial_time = 0;
    }
}