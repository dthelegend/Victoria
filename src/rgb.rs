use cortex_m::singleton;
use embedded_hal::digital::OutputPin;
use rp2040_hal::dma::{single_buffer, SingleChannel};
use rp2040_hal::gpio::{FunctionPio0, FunctionSio, Pin, PinId, PullDown, PullUp, SioOutput};
use rp2040_hal::pio::PinDir::Output;
use rp2040_hal::pio::{Buffers, InstallError, PIOExt, Running, ShiftDirection, StateMachine, StateMachineIndex, Stopped, Tx, UninitStateMachine, PIO};
use rp2040_hal::rom_data::reset_to_usb_boot;

pub struct RGBController<const NUMBER_OF_LEDS: usize, P : PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> {
    sm: StateMachine<(P, SM), Stopped>,
    tx: Tx<(P, SM)>,
    rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>
}

pub trait RGBPattern<const N : usize> {
    fn get_pattern(&self) -> &'static [Color; N];
}

#[derive(Copy, Clone)]
pub union Color {
    color_data : u32,
    color_bits : [u8; 4]
}

impl Default for Color {
    fn default() -> Self {
        Color::rgb(0,0,0)
    }
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        self.as_u32()
    }
}

impl Color {
    pub const fn as_u32(&self) -> u32 {
        unsafe { self.color_data }
    }
    
    pub const fn r(&self) -> &u8 {
        unsafe {
            &self.color_bits[2]
        }
    }
    pub const fn g(&self) -> &u8 {
        unsafe {
            &self.color_bits[1]
        }
    }
    pub const fn b(&self) -> &u8 {
        unsafe {
            &self.color_bits[3]
        }
    }
    
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color{ color_bits: [ g, r, b, 0] }
    }
}

impl <const N : usize, P : PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> RGBController<N, P, SM, RGBEnablePinId> {
    pub fn initialise<RGBDataPinID: PinId>(pio : &mut PIO<P>, uninit_sm: UninitStateMachine<(P, SM)>, rgb_data_pin: Pin<RGBDataPinID, FunctionPio0, PullDown>, mut rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>) -> RGBController<N, P, SM, RGBEnablePinId>  {
        let led_program = pio_proc::pio_asm!(
            ".define T1 3",
            ".define T2 3",
            ".define T3 5",
            ".side_set 1",
            ".wrap_target",
            "bitloop:",
                "out x 1            side 0  [T3 - 1]", // ensures that we pull low on stall
                "jmp !x do_one      side 1  [T1 - 1]",
            // do_zero:
                "jmp bitloop        side 0  [T2 - 1]",
            "do_one:",
                "nop                side 1  [T2 - 1]",
            ".wrap"
        );

        // Initialize and start PIO
        // TODO Handle
        let installed = pio.install(&led_program.program).unwrap();
        const SK6812_DATA_TRANSMISSION_UNIT_TIME : f32 = 32e-6;
        const CLK_DIVIDER : (u16, u8) = const {
            let a = SK6812_DATA_TRANSMISSION_UNIT_TIME * (super::XOSC_CRYSTAL_FREQ as f32) / 3f32;

            let a_floor = a as u16;
            let a_rem = a - (a_floor as f32);
            let a_rem_floor = (a_rem * 256f32) as u8;

            (a_floor, a_rem_floor)
        };

        let rgb_data_pin_id = rgb_data_pin.id().num;

        let (mut sm, _, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
            // Set clock speed
            .clock_divisor_fixed_point(CLK_DIVIDER.0, CLK_DIVIDER.1)
            // Shift out right as our data is laid out little-endian
            .out_shift_direction(ShiftDirection::Left)
            // Enable auto-pull
            .autopull(true)
            // Only pull 24 bits before auto-pulling
            .pull_threshold(24)
            // Set the pin which will be side_set
            .side_set_pin_base(rgb_data_pin_id)
            // We are only using the TX buffer
            .buffers(Buffers::OnlyTx)
            // build!
            .build(uninit_sm);

        sm.set_pindirs([(rgb_data_pin_id, Output)]);

        // TODO Handle
        rgb_enable_pin.set_low().unwrap();

        RGBController {
            sm,
            tx,
            rgb_enable_pin
        }
    }

    pub fn apply_pattern<T>(self, ch : T) -> (T, Self) where T : SingleChannel {
        let Self { sm, tx, mut rgb_enable_pin } = self;;

        let sm = sm.start();

        let pattern_message = [Color::rgb(255,255,255); 68];

        let tx_buf = singleton!(: [u32; 68] = pattern_message.map(|x| x.as_u32())).unwrap();

        let tx_transfer = single_buffer::Config::new(ch, tx_buf, tx).start();

        let (ch0, tx_buf, tx) = tx_transfer.wait();

        reset_to_usb_boot(0, 0);

        let sm = sm.stop();

        (ch0, Self {
            sm,
            tx,
            rgb_enable_pin
        })
    }
}

pub struct RGBControllerInTransition<const N : usize, P : PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> {
    sm: StateMachine<(P, SM), Running>,
    tx: Tx<(P, SM)>,
    rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>
}