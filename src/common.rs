use core::ops::{Div, Mul, Sub};
use cortex_m::prelude::_embedded_hal_timer_CountDown;
use rp2040_hal::timer::CountDown;

pub fn fixed_point_div<T, U, V, W>(dividend: T, divisor: U) -> (u16, u8)
where
    T: Div<U, Output = u32> + Sub<V, Output = W> + Copy,
    U: Mul<u32, Output = V> + Copy,
    W: Mul<u32>,
    W::Output: Div<U, Output = u32>,
{
    let int = dividend / divisor;
    let rem = dividend - (divisor * int);
    let frac = (rem * const { u8::MAX as u32 }) / divisor;

    (int as u16, frac as u8)
}

pub enum Assert<const CHECK: bool> {}

pub trait IsTrue {}

impl IsTrue for Assert<true> {}

pub struct ClampedTimer<'a> {
    timer: CountDown<'a>,
    is_clamped: bool,
    period: <CountDown<'a> as _embedded_hal_timer_CountDown>::Time,
}

impl<'a> ClampedTimer<'a> {
    pub fn new(
        timer: CountDown<'a>,
        period: impl Into<<CountDown<'a> as _embedded_hal_timer_CountDown>::Time>,
    ) -> Self {
        ClampedTimer {
            timer,
            is_clamped: true,
            period: period.into(),
        }
    }

    pub fn restart(&mut self) {
        self.timer.start(self.period);
        self.is_clamped = false;
    }

    pub fn wait(&mut self) -> bool {
        if !self.is_clamped {
            self.is_clamped = self.timer.wait().is_ok();
        }

        self.is_clamped
    }
}
