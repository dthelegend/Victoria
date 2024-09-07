#![no_std]
#![no_main]
#![feature(generic_const_exprs)]

mod common;
mod constants;
mod hal;
mod keyboard;
mod rgb;

use core::panic::PanicInfo;
use cortex_m::prelude::_embedded_hal_timer_CountDown;
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
use rp2040_hal::fugit::ExtU32;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{StringDescriptors, UsbDeviceBuilder, UsbVidPid};
use usb_device::UsbError;
use usbd_human_interface_device::descriptor::InterfaceProtocol;
use usbd_human_interface_device::device::keyboard::{
    NKROBootKeyboardConfig, NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR,
};
use usbd_human_interface_device::interface::{InterfaceBuilder, ManagedIdleInterfaceConfig};
use usbd_human_interface_device::prelude::UsbHidClassBuilder;
use usbd_human_interface_device::UsbHidError;

use keyboard::KeyboardInputManager;
use rgb::{RGBBufferManager, RGBController, RGBEffectResult};

use crate::common::ClampedTimer;
use crate::constants::{
    EFFECT_RATE, HID_TICK_RATE, KEYBOARD_POLLING_RATE, ROWS_PER_POLL, USB_ENDPOINT_POLL_RATE,
};
use crate::hal::entry;
use crate::keyboard::{BasicKeymap, KeyMap};
use crate::rgb::{RGBEffect, UnicornBarfCircleEffect};
use constants::RESET_DELAY;

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
        // RGBCycleEffect::new([Color::hsl(0x0, 0x0, u8::MAX / 32)]); // Less blinding
        // UnicornBarfCircleEffect::<{ u8::MAX }, 0xA, 0x0F>::new(); // 0x3F is already pretty bright; Also gets pretty stilted at < 0xF
        UnicornBarfWaveEffect::<{ u8::MAX }, 0xA, 0x0F>::new(); // 0x3F is already pretty bright; Also gets pretty stilted at < 0xF
        // StaticRGBEffect::<0,0,0>{}; // Turn it off

    effect.apply_effect(&mut buf_man);

    let active_controller = rgb_controller.start_effect(dma.ch0);

    let mut effect_timer = timer.count_down();
    let mut delay_timer = ClampedTimer::new(timer.count_down(), RESET_DELAY);

    let mut current_state = active_controller.start_pattern(buf_man).wait();

    //USB
    let usb_bus = UsbBusAllocator::new(rp2040_hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let config = NKROBootKeyboardConfig::new(ManagedIdleInterfaceConfig::new(
        InterfaceBuilder::new(NKRO_BOOT_KEYBOARD_REPORT_DESCRIPTOR)
            .unwrap()
            .description("It's the Daudboard. What more could you want?")
            .boot_device(InterfaceProtocol::Keyboard)
            .idle_default(500.millis())
            .unwrap()
            .in_endpoint(USB_ENDPOINT_POLL_RATE.into_duration())
            .unwrap()
            .with_out_endpoint(100.millis())
            .unwrap()
            .build(),
    ));

    let mut keyboard = UsbHidClassBuilder::new().add_device(config).build(&usb_bus);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .strings(&[StringDescriptors::default()
            .manufacturer("Daudi")
            .product("The Daudboard")
            .serial_number("1")])
        .unwrap()
        .build();

    let row_pin_group = (
        pins.row1.reconfigure(),
        pins.row2.reconfigure(),
        pins.row3.reconfigure(),
        pins.row4.reconfigure(),
        pins.row5.reconfigure(),
    );

    let col_pin_group = (
        pins.col1.reconfigure(),
        pins.col2.reconfigure(),
        pins.col3.reconfigure(),
        pins.col4.reconfigure(),
        pins.col5.reconfigure(),
        pins.col6.reconfigure(),
        pins.col7.reconfigure(),
        pins.col8.reconfigure(),
        pins.col9.reconfigure(),
        pins.col10.reconfigure(),
        pins.col11.reconfigure(),
        pins.col12.reconfigure(),
        pins.col13.reconfigure(),
        pins.col14.reconfigure(),
        pins.col15.reconfigure(),
    );

    let mut input_manager =
        KeyboardInputManager::initialise(row_pin_group, col_pin_group).activate();

    // Keyboard timers
    let mut tick_count_down = timer.count_down();
    let mut poll_timer = timer.count_down();

    effect_timer.start(EFFECT_RATE.into_duration());
    tick_count_down.start(HID_TICK_RATE.into_duration());
    poll_timer.start((KEYBOARD_POLLING_RATE * ROWS_PER_POLL).into_duration());

    loop {
        {
            // Check the keyboard input
            if poll_timer.wait().is_ok() {
                if let Some(key_buff_copy) = input_manager.continue_polling() {
                    match keyboard
                        .device()
                        .write_report(BasicKeymap::transform(key_buff_copy))
                    {
                        Ok(_) => {}
                        Err(UsbHidError::WouldBlock) => {}
                        Err(UsbHidError::Duplicate) => {}
                        Err(_) => panic!(),
                    }
                }
            }
        }

        {
            // Check the usb poller
            if usb_dev.poll(&mut [&mut keyboard]) {
                match keyboard.device().read_report() {
                    Err(UsbError::WouldBlock) => {
                        //do nothing
                    }
                    Err(e) => {
                        panic!("Failed to read keyboard report: {:?}", e)
                    }
                    Ok(_leds) => {
                        // TODO create an effect that can use this
                    }
                }
            }
        }

        {
            // Perform mandatory keyboard tick
            if tick_count_down.wait().is_ok() {
                match keyboard.tick() {
                    Err(UsbHidError::WouldBlock) => {}
                    Ok(_) => {}
                    Err(_) => panic!(),
                };
            }
        }

        {
            // Update the rgb
            current_state = match (delay_timer.wait(), current_state) {
                (true, RGBEffectResult::ShouldBlock(still_working)) => still_working.wait(),
                (true, RGBEffectResult::Finished(stalled, mut buf_man)) => {
                    delay_timer.restart();
                    if effect_timer.wait().is_ok() {
                        effect.apply_effect(&mut buf_man);
                    }

                    stalled.start_pattern(buf_man).wait()
                }
                (false, a) => a,
            }
        }
    }
}
