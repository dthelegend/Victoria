use rp2040_hal::fugit::{HertzU32, MicrosDurationU32};

pub const NUMBER_OF_LEDS: usize = 68;
pub const RESET_DELAY: MicrosDurationU32 = MicrosDurationU32::micros((60 * NUMBER_OF_LEDS) as u32);
pub const EFFECT_RATE: HertzU32 = HertzU32::nanos(500);

pub const POLLING_RATE: HertzU32 = HertzU32::Hz(100);
pub const HID_TICK_RATE: HertzU32 = HertzU32::millis(1);