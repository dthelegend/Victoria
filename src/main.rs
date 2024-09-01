#![no_std]
#![no_main]

mod constants;
mod rgb;

use core::default::Default;
use core::panic::PanicInfo;
use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::digital::PinState;
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::pio::PIOExt;
use rp2040_hal::{Clock, Watchdog};
use rp2040_hal::{entry, pac};

use crate::rgb::{Color, RGBBufferManager, RGBController, RGBCycleEffect, RGBEffect, RGBEffectResult, StalledRGBEffectController, UnicornBarfEffect, RESET_DELAY};
use rp2040_hal::dma::DMAExt;
use rp2040_hal::fugit::{ExtU32, Duration};
// use usb_device::class_prelude::*;

use constants::{RGBDataPin, RGBEnablePin};
use rp2040_hal::gpio::{Pin};

use rp2040_hal::rom_data::reset_to_usb_boot;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    reset_to_usb_boot(0,0);

    loop {}
}

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

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

    let rgb_enable_pin: Pin<RGBEnablePin, _, _> = pins.gpio26
        .into_pull_type()
        .into_push_pull_output_in_state(PinState::Low);
    let rgb_data_pin: Pin<RGBDataPin, _, _> = pins.gpio25
        .into_pull_type()
        .into_function();

    let rgb_controller = RGBController::initialise(&mut pio, sm0, rgb_data_pin, rgb_enable_pin, clocks.peripheral_clock.freq());

    let mut buf_man = RGBBufferManager::create();

    let mut effect =
        // RGBCycleEffect::new([Color::rgb(0x01, 0x0, 0x0), Color::rgb(0x00, 0x01, 0x0), Color::rgb(0x00, 0x0, 0x01)]); // R G B
        // RGBCycleEffect::new([Color::hex(0x8ACE00), Color::OFF]); // Brat summer
        // RGBCycleEffect::new([Color::WHITE, Color::OFF]); // IM BLINDED BY THE LIGHTS
        UnicornBarfEffect::<0xFF,0x01, 1>::new();

    effect.apply_effect(&mut buf_man);

    let active_controller = rgb_controller.start_effect(dma.ch0);

    let mut effect_timer = timer.count_down();

    // TODO replace with a more permanent solution
    let mut cycle_count = u8::MAX;
    let effect_timing = 1000.millis();


    let mut current_state = active_controller.start_pattern(buf_man).wait();

    effect_timer.start(effect_timing);
    loop {
        match current_state {
            RGBEffectResult::ShouldBlock(still_working) => {
                current_state = still_working.wait();
            },
            RGBEffectResult::Finished(stalled, mut buf_man) => {
                let mut delay_timer = timer.count_down();

                delay_timer.start(RESET_DELAY);

                if effect_timer.wait().is_ok() {
                    effect.apply_effect(&mut buf_man);
                    effect_timer.start(effect_timing);
                    cycle_count -= 1;
                    if cycle_count == 0 {
                        panic!();
                    }
                }

                nb::block!(delay_timer.wait());

                current_state = stalled.start_pattern(buf_man).wait();
            }
        }
    }
}
