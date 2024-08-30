use cortex_m::singleton;
use embedded_hal::digital::OutputPin;
use pio::ProgramWithDefines;
use rp2040_hal::dma::single_buffer::Transfer;
use rp2040_hal::dma::{single_buffer, SingleChannel};
use rp2040_hal::gpio::{FunctionPio0, FunctionSio, Pin, PinId, PullDown, PullUp, SioOutput};
use rp2040_hal::pio::PinDir::Output;
use rp2040_hal::pio::{
    PIOExt, Running, ShiftDirection, StateMachine, StateMachineIndex, Stopped, Tx,
    UninitStateMachine, PIO,
};

const NUMBER_OF_LEDS: usize = 68;

pub struct RGBController<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> {
    rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>,
    sm: StateMachine<(P, SM), Stopped>,
    tx: Tx<(P, SM)>,
}

#[derive(Copy, Clone)]
pub union Color {
    color_data: u32,
    color_bits: [u8; 4],
}

impl Default for Color {
    fn default() -> Self {
        Color::rgb(0, 0, 0)
    }
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        self.as_u32()
    }
}

impl Color {
    const BITS: usize = 24;

    pub const fn as_u32(&self) -> u32 {
        unsafe { self.color_data }
    }

    pub const fn r(&self) -> &u8 {
        unsafe { &self.color_bits[2] }
    }
    pub const fn g(&self) -> &u8 {
        unsafe { &self.color_bits[3] }
    }
    pub const fn b(&self) -> &u8 {
        unsafe { &self.color_bits[1] }
    }

    pub fn r_mut(&mut self) -> &mut u8 {
        unsafe { &mut self.color_bits[2] }
    }
    pub fn g_mut(&mut self) -> &mut u8 {
        unsafe { &mut self.color_bits[3] }
    }
    pub fn b_mut(&mut self) -> &mut u8 {
        unsafe { &mut self.color_bits[1] }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            color_bits: [0, b, r, g],
        }
    }
}

impl<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> RGBController<P, SM, RGBEnablePinId> {
    pub fn initialise<RGBDataPinID: PinId>(
        pio: &mut PIO<P>,
        uninit_sm: UninitStateMachine<(P, SM)>,
        rgb_data_pin: Pin<RGBDataPinID, FunctionPio0, PullDown>,
        mut rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>,
    ) -> RGBController<P, SM, RGBEnablePinId> {

        struct ExpandedDefines { T3: i32, T1: i32, T2: i32 }

        let led_program = pio_proc::pio_asm!(
            ".define public T1 3",
            ".define public T2 3",
            ".define public T3 5",
            ".side_set 1",
            ".wrap_target",
            "bitloop:",
            "out x 1            side 0  [T3 - 1]", // ensures that we pull low on stall
            "jmp !x do_zero     side 1  [T1 - 1]",
            // do_one:
            "jmp bitloop        side 1  [T2 - 1]",
            "do_zero:",
            "nop                side 0  [T2 - 1]",
            ".wrap"
        );

        // Initialize and start PIO
        // TODO Handle
        let installed = pio.install(&led_program.program).unwrap();

        /// The frequency of 1 bit signals
        const DATA_TRANSMISSION_FREQ: f32 = 1.0 / 1.28e-6;
        const CYCLES_PER_UNIT: f32 = led_program.public_defines.T1 + led_program.public_defines.T2 + led_program.public_defines.T3;
        const CLK_DIVIDER: (u16, u8) = const {
            let a = super::XOSC_CRYSTAL_FREQ as f32 / (DATA_TRANSMISSION_FREQ * CYCLES_PER_UNIT);

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
            // Only pull the number of color bits before auto-pulling
            .pull_threshold(Color::BITS as u8)
            // Set the pin which will be side_set
            .side_set_pin_base(rgb_data_pin_id)
            // build!
            .build(uninit_sm);

        sm.set_pindirs([(rgb_data_pin_id, Output)]);

        // Ensure the pin is high just in case
        rgb_enable_pin.set_high().unwrap();

        RGBController {
            sm,
            tx,
            rgb_enable_pin,
        }
    }

    pub fn start_pattern<CH0: SingleChannel>(
        self,
        ch0: CH0,
        effect: impl Effect,
    ) -> (
        RGBEffectController<P, SM, RGBEnablePinId, CH0>,
        impl EffectModifier,
    ) {
        let Self {
            sm,
            tx,
            mut rgb_enable_pin,
        } = self;

        // Pull low to enable RGB
        rgb_enable_pin.set_low().unwrap();

        let sm = sm.start();

        let (effect_mod, tx_effect_buff) = effect.split();

        // Create the initial transfer
        let tx_transfer = single_buffer::Config::new(ch0, tx_effect_buff.0, tx).start();

        (
            RGBEffectController {
                sm,
                rgb_enable_pin,
                tx_transfer,
            },
            effect_mod,
        )
    }
}

pub trait PatternGenerator {
    fn next_pattern();
}

// An object that holds an RGB Controller with its current state.
pub struct RGBEffectController<
    P: PIOExt,
    SM: StateMachineIndex,
    RGBEnablePinId: PinId,
    CH: SingleChannel,
> {
    rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>,
    sm: StateMachine<(P, SM), Running>,
    // Even though not enforced, the effect controller is implied to 'own' these buffers
    // If there are multiple RGBEffectControllers they would have to share
    tx_transfer: Transfer<CH, &'static [u32; NUMBER_OF_LEDS], Tx<(P, SM)>>,
}

impl<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId, CH: SingleChannel>
    RGBEffectController<P, SM, RGBEnablePinId, CH>
{
    pub fn cancel(self) -> (RGBController<P, SM, RGBEnablePinId>, CH) {
        let Self {
            sm,
            mut rgb_enable_pin,
            tx_transfer,
        } = self;

        // We bin the effect
        let (ch0, _, tx) = tx_transfer.wait();

        rgb_enable_pin.set_low().unwrap();

        let sm = sm.stop();

        (
            RGBController {
                sm,
                tx,
                rgb_enable_pin,
            },
            ch0,
        )
    }

    pub fn next(self) -> (Self, bool) {
        let Self {
            rgb_enable_pin,
            sm,
            tx_transfer,
        } = self;
        if tx_transfer.is_done() {
            // Should not block, but provides safety in case it does
            let (ch0, tx_buf, tx) = tx_transfer.wait();

            // Start next transfer
            let tx_transfer = single_buffer::Config::new(ch0, tx_buf, tx).start();

            (
                Self {
                    rgb_enable_pin,
                    sm,
                    tx_transfer,
                },
                true,
            )
        } else {
            (
                Self {
                    rgb_enable_pin,
                    sm,
                    tx_transfer,
                },
                false,
            )
        }
    }
}

// TODO StalledRGBController

pub trait EffectModifier {
    fn step_effect(self) -> Self;
}

pub trait Effect {
    fn split(self) -> (impl EffectModifier, EffectBuffer);
}

pub struct EffectBuffer(&'static [u32; NUMBER_OF_LEDS]);

pub struct StaticRGBEffect<const R: u8, const G: u8, const B: u8>(); // zst

impl<const R: u8, const G: u8, const B: u8> Effect for StaticRGBEffect<R, G, B> {
    fn split(self) -> (impl EffectModifier, EffectBuffer) {
        let buffer =
            singleton!(: [u32; NUMBER_OF_LEDS] = [Color::rgb(R,G,B).as_u32(); NUMBER_OF_LEDS])
                .unwrap();

        (StaticRGBEffectModifier(), EffectBuffer(buffer))
    }
}

pub struct StaticRGBEffectModifier();

impl EffectModifier for StaticRGBEffectModifier {
    // Purposely the identity function for the static effect
    fn step_effect(self) -> Self {
        self
    }
}
