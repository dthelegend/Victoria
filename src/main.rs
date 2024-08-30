#![no_std]
#![no_main]

mod rgb;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin, PinState};
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::pio::PIOExt;
use rp2040_hal::rom_data::reset_to_usb_boot;
use rp2040_hal::Watchdog;
use rp2040_hal::{entry, pac};
use rp2040_hal::timer::CountDown;

use crate::rgb::{EffectModifier, RGBController, StaticRGBEffect};
use rp2040_hal::dma::DMAExt;
use rp2040_hal::fugit::ExtU32;
use usb_device::class_prelude::*;

use panic_halt as _;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

#[repr(u8)]
enum PinMap {
    Col13 = 0,
    Col12 = 1,
    Col11 = 2,
    Col10 = 3,
    Col9 = 4,
    Col8 = 5,
    Col7 = 6,
    Col6 = 7,
    Col5 = 8,
    Col4 = 9,
    Col3 = 10,
    Col2 = 11,
    Col15 = 12,
    Col14 = 13,

    Row4 = 14,
    Row5 = 15,

    Col1 = 22,
    Row3 = 23,
    Row1 = 24,
    RGBData = 25,
    RGBEnable = 26,
    Row2 = 27,
}

impl Into<u8> for PinMap {
    fn into(self) -> u8 {
        self as u8
    }
}

const fn pin_to_mask(a: PinMap) -> u32 {
    0x1u32 << (a as u8)
}

const COL_MASK: u32 = {
    use PinMap::*;

    pin_to_mask(Col1)
        | pin_to_mask(Col2)
        | pin_to_mask(Col3)
        | pin_to_mask(Col4)
        | pin_to_mask(Col5)
        | pin_to_mask(Col6)
        | pin_to_mask(Col7)
        | pin_to_mask(Col8)
        | pin_to_mask(Col9)
        | pin_to_mask(Col10)
        | pin_to_mask(Col11)
        | pin_to_mask(Col12)
        | pin_to_mask(Col13)
        | pin_to_mask(Col14)
        | pin_to_mask(Col15)
};

const ROW_MASK: u32 = {
    use PinMap::*;

    pin_to_mask(Row1)
        | pin_to_mask(Row2)
        | pin_to_mask(Row3)
        | pin_to_mask(Row4)
        | pin_to_mask(Row5)
};

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

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

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

    let rgb_enable_pin = pins
        .gpio26
        .into_pull_type::<_>()
        .into_push_pull_output_in_state(PinState::High);
    let rgb_data_pin = pins.gpio25.into_function();

    let rgb_controller = RGBController::initialise(&mut pio, sm0, rgb_data_pin, rgb_enable_pin);

    let (mut static_effect_controller, mut effect_modifier) =
        rgb_controller.start_pattern(dma.ch0, StaticRGBEffect::<255, 255, 255>());

    let mut total_time_count_down = timer.count_down();

    let mut delay_timer = timer.count_down();

    total_time_count_down.start(10.secs());

    while !total_time_count_down.wait().is_ok() {
        effect_modifier = effect_modifier.step_effect();
        let (new_controller, transfer_complete) = static_effect_controller.next();
        static_effect_controller = new_controller;

        if transfer_complete {
            delay_timer.start(80.micros());

            while !delay_timer.wait().is_ok() {}
        }
    }

    reset_to_usb_boot(0, 0);

    loop {}
}
