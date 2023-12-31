use feather_m4::hal;
use feather_m4::pac;

use fugit::HertzU32 as Hertz;
use hal::prelude::*;

use hal::clock::GenericClockController;
use hal::timer::TimerCounter;

use pac::interrupt;
use pac::NVIC;

use core::sync::atomic::{AtomicUsize, Ordering};

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
        // TODO: Consider setting prority to 0 so that no timer interrupts are missed
        nvic.set_priority(interrupt::TC2, 0);
        NVIC::unmask(interrupt::TC2);
    }

    timer.start(Hertz::kHz(1).into_duration());
    timer.enable_interrupt();
}

static mut ELAPSED_MS: AtomicUsize = AtomicUsize::new(0);

pub fn elapsed_ms() -> usize {
    unsafe { ELAPSED_MS.load(Ordering::Relaxed) }
}

#[interrupt]
fn TC2() {
    unsafe {
        // TODO: Don't steal the peripherials!!
        let tc2 = feather_m4::pac::Peripherals::steal().TC2;
        // This flag needs to be cleared (by writing 1 to the bit) or else the interrupt will never exit
        tc2.count16().intflag.write(|w| w.ovf().set_bit());

        ELAPSED_MS.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct UpTimer {
    initial_time: usize,
}

impl UpTimer {
    pub fn new() -> Self {
        Self {
            initial_time: elapsed_ms(),
        }
    }

    pub fn elapsed_ms(&self) -> usize {
        elapsed_ms() - self.initial_time
    }

    pub fn reset(&mut self) {
        self.initial_time = elapsed_ms();
    }
}

// impl embedded_hal::timer::CountDown for UpTimer {
//     type Time = u64;

//     fn start<T>(&mut self, count: T)
//     where
//         T: Into<Self::Time> {
//         todo!()
//     }

//     fn wait(&mut self) -> nb::Result<(), Void> {
//         todo!()
//     }
// }
