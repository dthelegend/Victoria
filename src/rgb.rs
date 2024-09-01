use cortex_m::singleton;
use defmt::export::u8;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use rp2040_hal::dma::single_buffer::Transfer;
use rp2040_hal::dma::{single_buffer, SingleChannel};
use rp2040_hal::fugit::{HertzU32, MicrosDurationU32};
use rp2040_hal::gpio::{FunctionPio0, FunctionSio, Pin, PinId, PullUp, SioOutput};
use rp2040_hal::pio::PinDir::Output;
use rp2040_hal::pio::{
    PIOExt, Running, ShiftDirection, StateMachine, StateMachineIndex, Stopped, Tx,
    UninitStateMachine, PIO,
};

const NUMBER_OF_LEDS: usize = 68;
pub const RESET_DELAY : MicrosDurationU32 = MicrosDurationU32::micros(60 * (NUMBER_OF_LEDS as u32));

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

    pub const WHITE : Color = Color { color_data : u32::MAX };

    pub const OFF : Color = Color { color_data : u32::MIN };

    pub const fn hex(code :u32) -> Color {
        Color::rgb((code >> u8::BITS * 2) as u8, (code >> u8::BITS) as u8, code as u8)
    }

    // u8 (0 - 255) -> (0 - 1)
    pub const fn hsl(h: u16, s: u8, l: u8) -> Color {

        // Chroma calculation: C = (1 - |2L - 1|) * S
        let c = (((u8::MAX - 2 * l.abs_diff(u8::MAX >> 1)) as u16 * s as u16) / 255) as u8;

        // X calculation: X = C * (1 - |(H / 60) % 2 - 1|)
        let x = ((c as u32 * (u16::MAX as u32 - ((h as u32 * 6 % ((u16::MAX as u32) << 1)).abs_diff(u16::MAX as u32)))) / u16::MAX as u32) as u8;

        // Lightness match value
        let m = l.saturating_sub(c / 2);

        // Sector definitions using bounds
        const DIV1: u16 = u16::MAX / 6;                       // 256 / 6;
        const DIV2: u16 = ((u16::MAX as u32 * 2) / 6) as u16; // 2 * 256 / 6
        const DIV3: u16 = ((u16::MAX as u32 * 3) / 6) as u16; // 3 (256 / 6)
        const DIV4: u16 = ((u16::MAX as u32 * 4) / 6) as u16; // 4 * (256 / 6)
        const DIV5: u16 = ((u16::MAX as u32 * 5) / 6) as u16; // 5 * (256 / 6)

        // Determine RGB components based on hue sector
        let (r_prime, g_prime, b_prime) = match h {
            0..=DIV1 => (c, x, 0),           // Red to yellow
            DIV1..=DIV2 => (x, c, 0),        // Yellow to green
            DIV2..=DIV3 => (0, c, x),        // Green to cyan
            DIV3..=DIV4 => (0, x, c),        // Cyan to blue
            DIV4..=DIV5 => (x, 0, c),        // Blue to magenta
            _ => (c, 0, x),                  // Magenta to red
        };

        // Combine components and adjust for lightness
        Self::rgb(
            r_prime.saturating_add(m),
            g_prime.saturating_add(m),
            b_prime.saturating_add(m),
        )
    }
}

impl<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId> RGBController<P, SM, RGBEnablePinId> {
    pub fn initialise<RGBDataPinId: PinId>(
        pio: &mut PIO<P>,
        uninit_sm: UninitStateMachine<(P, SM)>,
        rgb_data_pin: Pin<RGBDataPinId, FunctionPio0, PullUp>,
        mut rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>,
        clock_freq: HertzU32
    ) -> RGBController<P, SM, RGBEnablePinId> {
        let led_program = pio_proc::pio_asm!(
            ".define public T1 2",
            ".define public T2 5",
            ".define public T3 3",
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
        let installed = pio.install(&led_program.program).unwrap();

        /// The frequency of 1 bit signals
        const DATA_TRANSMISSION_FREQ: HertzU32 = HertzU32::kHz(800);
        // should be const at O3
        let CYCLES_PER_UNIT: u32 = (led_program.public_defines.T1
            + led_program.public_defines.T2
            + led_program.public_defines.T3) as u32;
        let CLK_DIVIDER: (u16, u8) = {
            let bit_freq = DATA_TRANSMISSION_FREQ * CYCLES_PER_UNIT;

            let int = clock_freq / bit_freq;
            let rem = clock_freq - (int * bit_freq);
            let frac = (rem * const { 1u32 << u8::BITS }) / bit_freq;

            (int as u16, frac as u8)
        };

        let rgb_data_pin_id = rgb_data_pin.id().num;

        let (mut sm, _, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
            // Set buffers
            .buffers(rp2040_hal::pio::Buffers::OnlyTx)
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
        if !rgb_enable_pin.is_set_high().unwrap() {
            rgb_enable_pin.set_high().unwrap();
        }

        RGBController {
            sm,
            tx,
            rgb_enable_pin,
        }
    }

    pub fn start_effect<CH0: SingleChannel>(
        self,
        ch: CH0,
    ) ->
        StalledRGBEffectController<P, SM, RGBEnablePinId, CH0> {
        let Self {
            sm,
            tx,
            mut rgb_enable_pin,
        } = self;

        // Pull high to enable RGB
        rgb_enable_pin.set_low().unwrap();

        let sm = sm.start();

        StalledRGBEffectController {
            sm,
            rgb_enable_pin,
            ch,
            tx
        }
    }
}

pub enum RGBEffectResult<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId, CH: SingleChannel>
{
    ShouldBlock(RGBEffectController<P, SM, RGBEnablePinId, CH>),
    Finished(StalledRGBEffectController<P, SM, RGBEnablePinId, CH>, RGBBufferManager),
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
    tx_transfer: Transfer<CH, &'static mut [u32; NUMBER_OF_LEDS], Tx<(P, SM)>>,
}

impl<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId, CH: SingleChannel>
    RGBEffectController<P, SM, RGBEnablePinId, CH>
{
    pub fn wait(self) -> RGBEffectResult<P, SM, RGBEnablePinId, CH> {
        let Self {
            rgb_enable_pin,
            sm,
            tx_transfer,
        } = self;
        if tx_transfer.is_done() {
            // Should not block, but provides safety in case it does
            let (ch, tx_buf, tx) = tx_transfer.wait();

            RGBEffectResult::Finished(StalledRGBEffectController {
                rgb_enable_pin,
                sm,
                ch,
                tx,
            }, RGBBufferManager { buffer: tx_buf })
        } else {
            RGBEffectResult::ShouldBlock(Self {
                rgb_enable_pin,
                sm,
                tx_transfer,
            })
        }
    }
}

pub struct StalledRGBEffectController<
    P: PIOExt,
    SM: StateMachineIndex,
    RGBEnablePinId: PinId,
    CH: SingleChannel,
> {
    rgb_enable_pin: Pin<RGBEnablePinId, FunctionSio<SioOutput>, PullUp>,
    sm: StateMachine<(P, SM), Running>,
    ch: CH,
    tx: Tx<(P, SM)>,
}

impl<P: PIOExt, SM: StateMachineIndex, RGBEnablePinId: PinId, CH: SingleChannel>
    StalledRGBEffectController<P, SM, RGBEnablePinId, CH>
{

    pub fn cancel(self) -> (RGBController<P, SM, RGBEnablePinId>, CH) {
        let Self {
            sm,
            rgb_enable_pin,
            ch,
            tx,
        } = self;

        let sm = sm.stop();

        (
            RGBController {
                sm,
                tx,
                rgb_enable_pin,
            },
            ch,
        )
    }

    pub fn start_pattern(self, rgbbuffer_manager: RGBBufferManager) -> RGBEffectController<P, SM, RGBEnablePinId, CH> {
        let Self {
            sm,
            rgb_enable_pin,
            ch,
            tx,
        } = self;

        let RGBBufferManager {
            buffer
        } = rgbbuffer_manager;

        RGBEffectController {
            sm,
            rgb_enable_pin,
            tx_transfer: single_buffer::Config::new(ch, buffer, tx).start(),
        }
    }
}

pub struct RGBBufferManager {
    buffer: &'static mut [u32; NUMBER_OF_LEDS]
}

impl RGBBufferManager {
    pub fn fill(&mut self, color: Color) {
        self.buffer.fill(color.as_u32());
    }

    pub fn create() -> Self {
        let buffer = singleton!(: [u32; NUMBER_OF_LEDS] = [0; NUMBER_OF_LEDS]).unwrap();

        Self { buffer }
    }
}

pub trait RGBEffect {
    fn apply_effect(&mut self, buffer: &mut RGBBufferManager);
}

pub struct RGBCycleEffect<const N : usize> {
    colors: [Color; N],
    selector: usize
}

impl <const N : usize> RGBCycleEffect<N> {
    pub fn new(colors: [Color; N]) -> Self {
        Self {
            colors,
            selector: 0
        }
    }
}

impl <const N : usize> RGBEffect for RGBCycleEffect<N> {
    fn apply_effect(&mut self, buffer: &mut RGBBufferManager) {
        buffer.fill(self.colors[self.selector]);

        self.selector += 1;

        // enforce selector invariant
        if self.selector >= N {
            self.selector = 0;
        }
    }
}

pub struct UnicornBarfEffect<const S : u8, const L: u8, const STEP: u16> {
    current_hue: u16
}

impl <const S : u8, const L: u8, const STEP: u16> UnicornBarfEffect<S, L, STEP> {
    pub fn new() -> Self {
        UnicornBarfEffect {
            current_hue: 0
        }
    }
}

impl <const S : u8, const L: u8, const STEP: u16> RGBEffect for UnicornBarfEffect<S, L, STEP> {
    fn apply_effect(&mut self, buffer: &mut RGBBufferManager) {
        buffer.fill(Color::hsl(self.current_hue, S, L));

        self.current_hue = self.current_hue.checked_add(STEP).unwrap_or(0);
    }
}

pub struct StaticRGBEffect<const R: u8, const G: u8, const B: u8> {}

impl <const R: u8, const G: u8, const B: u8> RGBEffect for StaticRGBEffect<R, G, B> {
    fn apply_effect(&mut self, buffer: &mut RGBBufferManager) {
        buffer.fill(Color::rgb(R, G, B));
    }
}
