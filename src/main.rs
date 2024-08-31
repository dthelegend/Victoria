#![no_std]
#![no_main]

mod constants;
mod rgb;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::digital::PinState;
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::pio::PIOExt;
use rp2040_hal::rom_data::reset_to_usb_boot;
use rp2040_hal::Watchdog;
use rp2040_hal::{entry, pac};

use crate::rgb::{EffectModifier, RGBController, RGBEffectResult, StaticRGBEffect};
use rp2040_hal::dma::DMAExt;
use rp2040_hal::fugit::ExtU32;
use usb_device::class_prelude::*;

use constants::{RGBDataPin, RGBEnablePin};
use rp2040_hal::gpio::Pin;

#[allow(unused_imports)]
use panic_halt as _;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = rp2040_hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let dma = pac.DMA.split(&mut pac.RESETS);

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let rgb_enable_pin: Pin<RGBEnablePin, _, _> = pins
        .gpio26
        .into_pull_type::<_>()
        .into_push_pull_output_in_state(PinState::High);
    let rgb_data_pin: Pin<RGBDataPin, _, _> = pins.gpio25.into_function();

    let rgb_controller = RGBController::initialise(&mut pio, sm0, rgb_data_pin, rgb_enable_pin);

    let (static_effect_controller, mut effect_modifier) =
        rgb_controller.set_pattern(dma.ch0, StaticRGBEffect::<255, 255, 255>());

    let mut total_time_count_down = timer.count_down();

    let mut delay_timer = timer.count_down();

    total_time_count_down.start(10.secs());

    let mut current_state = static_effect_controller.start_pattern().wait();

    while !total_time_count_down.wait().is_ok() {
        match current_state {
            RGBEffectResult::ShouldBlock(still_working) => current_state = still_working.wait(),
            RGBEffectResult::Finished(stalled) => {
                delay_timer.start(80.micros());

                effect_modifier.step_effect();

                while !delay_timer.wait().is_ok() {}

                current_state = stalled.start_pattern().wait()
            }
        }
    }

    reset_to_usb_boot(0, 0);

    loop {}
}
