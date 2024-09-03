#![no_std]
#![no_main]
#![feature(gen_blocks)]

mod common;
mod constants;
mod hal;
mod keyboard;
mod rgb;

use core::panic::PanicInfo;
use cortex_m::prelude::_embedded_hal_timer_CountDown;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{StringDescriptors, UsbDeviceBuilder, UsbVidPid};
use usb_device::UsbError;
use usbd_human_interface_device::prelude::UsbHidClassBuilder;
use usbd_human_interface_device::UsbHidError;
use hal::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        dma::DMAExt,
        pac,
        pio::PIOExt,
        rom_data::reset_to_usb_boot,
        watchdog::Watchdog,
        Sio,
    },
    XOSC_CRYSTAL_FREQ,
};

use keyboard::KeyboardInputManager;
use rgb::{Color, RGBBufferManager, RGBController, RGBCycleEffect, RGBEffect, RGBEffectResult};

use crate::hal::entry;
use constants::RESET_DELAY;
use crate::constants::{EFFECT_RATE, HID_TICK_RATE, POLLING_RATE};
use crate::keyboard::BasicKeymap;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    reset_to_usb_boot(0, 0);

    loop {}
}

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

    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let dma = pac.DMA.split(&mut pac.RESETS);

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let rgb_controller = RGBController::initialise(
        &mut pio,
        sm0,
        pins.rgb_data.into_function().into_pull_type(),
        pins.rgb_enable.into_function().into_pull_type(),
        clocks.peripheral_clock.freq(),
    );

    let mut buf_man = RGBBufferManager::create();

    let mut effect =
        // RGBCycleEffect::new([Color::rgb(0x01, 0x0, 0x0), Color::rgb(0x00, 0x01, 0x0), Color::rgb(0x00, 0x0, 0x01)]); // R G B
        // StaticRGBEffect::<0x8A,0xCE,0x00>{}; // Brat summer
        // StaticRGBEffect::<0xFF,0xFF,0xFF>{}; // IM BLINDED BY THE LIGHTS
        RGBCycleEffect::new([Color::hsl(0x0, 0x0, u8::MAX / 32)]); // Less blinding
                                                                   // UnicornBarfEffect::<{ u16::MAX }, 0x2FFFFFFF, 0x0F>::new(); // 0x3F is already pretty bright; Also gets pretty stilted at < 0xF
                                                                   // StaticRGBEffect::<0,0,0>{}; // Turn it off

    effect.apply_effect(&mut buf_man);

    let active_controller = rgb_controller.start_effect(dma.ch0);

    let mut effect_timer = timer.count_down();

    let mut current_state = active_controller.start_pattern(buf_man).wait();

    //USB
    let usb_bus = UsbBusAllocator::new(rp2040_hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut keyboard = UsbHidClassBuilder::new()
        .add_device(
            usbd_human_interface_device::device::keyboard::NKROBootKeyboardConfig::default(),
        )
        .build(&usb_bus);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .strings(&[StringDescriptors::default()
            .manufacturer("usbd-human-interface-device")
            .product("The Daudboard")
            .serial_number("1")])
        .unwrap()
        .build();

    let mut input_manager = KeyboardInputManager::initialise(
        (
            pins.row1.into_function().into_pull_type(),
            pins.row2.into_function().into_pull_type(),
            pins.row3.into_function().into_pull_type(),
            pins.row4.into_function().into_pull_type(),
            pins.row5.into_function().into_pull_type(),
        ),
        (
            pins.col1.into_function().into_pull_type(),
            pins.col2.into_function().into_pull_type(),
            pins.col3.into_function().into_pull_type(),
            pins.col4.into_function().into_pull_type(),
            pins.col5.into_function().into_pull_type(),
            pins.col6.into_function().into_pull_type(),
            pins.col7.into_function().into_pull_type(),
            pins.col8.into_function().into_pull_type(),
            pins.col9.into_function().into_pull_type(),
            pins.col10.into_function().into_pull_type(),
            pins.col11.into_function().into_pull_type(),
            pins.col12.into_function().into_pull_type(),
            pins.col13.into_function().into_pull_type(),
            pins.col14.into_function().into_pull_type(),
            pins.col15.into_function().into_pull_type(),
        ),
    )
    .activate_with_keymap(BasicKeymap());

    // Keyboard timers
    let mut tick_count_down = timer.count_down();
    let mut poll_timer = timer.count_down();

    effect_timer.start(EFFECT_RATE.into_duration());
    tick_count_down.start(HID_TICK_RATE.into_duration());
    poll_timer.start(POLLING_RATE.into_duration());

    loop {
        if poll_timer.wait().is_ok() {
            match keyboard.device().write_report(input_manager.get_report()) {
                Ok(_) => {}
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Err(_) => panic!(),
            }
        }

        // Perform mandatory keyboard tick
        if tick_count_down.wait().is_ok() {
            match keyboard.tick() {
                Err(UsbHidError::WouldBlock) => {}
                Ok(_) => {}
                Err(_) => panic!(),
            };
        }

        // Write to the usb when it's ready
        if usb_dev.poll(&mut [&mut keyboard]) {
            match keyboard.device().read_report() {
                Err(UsbError::WouldBlock) => {
                    //do nothing
                }
                Err(e) => {
                    core::panic!("Failed to read keyboard report: {:?}", e)
                }
                Ok(leds) => {
                    // TODO
                }
            }
        }

        // Check RGB
        match current_state {
            RGBEffectResult::ShouldBlock(still_working) => {
                current_state = still_working.wait();
            }
            RGBEffectResult::Finished(stalled, mut buf_man) => {
                let mut delay_timer = timer.count_down();

                delay_timer.start(RESET_DELAY);

                if effect_timer.wait().is_ok() {
                    effect.apply_effect(&mut buf_man);
                }

                nb::block!(delay_timer.wait()).unwrap();

                current_state = stalled.start_pattern(buf_man).wait();
            }
        }
    }
}
