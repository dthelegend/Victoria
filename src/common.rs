use core::ops::{Div, Mul, Sub};

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
