use rp2040_hal::fugit::{HertzU32, MicrosDurationU32};

pub const NUMBER_OF_LEDS: usize = 68;
pub const RESET_DELAY: MicrosDurationU32 = MicrosDurationU32::micros((60 * NUMBER_OF_LEDS) as u32);
pub const EFFECT_RATE: HertzU32 = HertzU32::nanos(500);

//
pub const USB_ENDPOINT_POLL_RATE: HertzU32 = HertzU32::Hz(1000);
pub const KEYBOARD_POLLING_RATE: HertzU32 = HertzU32::Hz(4000);
pub const ROWS_PER_POLL: u32 = 4;
pub const HID_TICK_RATE: HertzU32 = HertzU32::millis(1);
