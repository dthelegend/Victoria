use crate::common::fixed_point_div;
use pio_proc::pio_asm;
use rp2040_hal::fugit::HertzU32;
use rp2040_hal::gpio::{AnyPin, FunctionPio0, PullUp};
use rp2040_hal::pio::PinDir::{Input, Output};
use rp2040_hal::pio::{PIOExt, StateMachine, StateMachineIndex, Stopped, UninitStateMachine, PIO};

// Define a tuple type for rows and columns to manage multiple pin types
type RowsTuple<IRow1, IRow2, IRow3, IRow4, IRow5> = (IRow1, IRow2, IRow3, IRow4, IRow5);
type ColsTuple<
    ICol1,
    ICol2,
    ICol3,
    ICol4,
    ICol5,
    ICol6,
    ICol7,
    ICol8,
    ICol9,
    ICol10,
    ICol11,
    ICol12,
    ICol13,
    ICol14,
    ICol15,
> = (
    ICol1,
    ICol2,
    ICol3,
    ICol4,
    ICol5,
    ICol6,
    ICol7,
    ICol8,
    ICol9,
    ICol10,
    ICol11,
    ICol12,
    ICol13,
    ICol14,
    ICol15,
);

struct KeyboardInputManager<
    P: PIOExt,
    SM: StateMachineIndex,
    IRow1: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    IRow2: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    IRow3: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    IRow4: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    IRow5: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol1: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol2: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol3: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol4: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol5: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol6: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol7: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol8: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol9: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol10: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol11: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol12: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol13: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol14: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    ICol15: AnyPin<Function = FunctionPio0, Pull = PullUp>,
> {
    sm: StateMachine<(P, SM), Stopped>,
    _rows: RowsTuple<IRow1, IRow2, IRow3, IRow4, IRow5>,
    _cols: ColsTuple<
        ICol1,
        ICol2,
        ICol3,
        ICol4,
        ICol5,
        ICol6,
        ICol7,
        ICol8,
        ICol9,
        ICol10,
        ICol11,
        ICol12,
        ICol13,
        ICol14,
        ICol15,
    >,
    column_mask: u32,
}

impl<
        P: PIOExt,
        SM: StateMachineIndex,
        IRow1: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        IRow2: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        IRow3: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        IRow4: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        IRow5: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol1: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol2: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol3: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol4: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol5: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol6: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol7: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol8: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol9: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol10: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol11: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol12: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol13: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol14: AnyPin<Function = FunctionPio0, Pull = PullUp>,
        ICol15: AnyPin<Function = FunctionPio0, Pull = PullUp>,
    >
    KeyboardInputManager<
        P,
        SM,
        IRow1,
        IRow2,
        IRow3,
        IRow4,
        IRow5,
        ICol1,
        ICol2,
        ICol3,
        ICol4,
        ICol5,
        ICol6,
        ICol7,
        ICol8,
        ICol9,
        ICol10,
        ICol11,
        ICol12,
        ICol13,
        ICol14,
        ICol15,
    >
{
    pub fn initialise(
        pio: &mut PIO<P>,
        uninit_sm: UninitStateMachine<(P, SM)>,
        clock_freq: HertzU32,
        rows: RowsTuple<IRow1, IRow2, IRow3, IRow4, IRow5>,
        cols: ColsTuple<
            ICol1,
            ICol2,
            ICol3,
            ICol4,
            ICol5,
            ICol6,
            ICol7,
            ICol8,
            ICol9,
            ICol10,
            ICol11,
            ICol12,
            ICol13,
            ICol14,
            ICol15,
        >,
    ) -> Self {
        // Convert pins into concrete types
        let concrete_rows = (
            rows.0.into(),
            rows.1.into(),
            rows.2.into(),
            rows.3.into(),
            rows.4.into(),
        );

        let concrete_cols = (
            cols.0.into(),
            cols.1.into(),
            cols.2.into(),
            cols.3.into(),
            cols.4.into(),
            cols.5.into(),
            cols.6.into(),
            cols.7.into(),
            cols.8.into(),
            cols.9.into(),
            cols.10.into(),
            cols.11.into(),
            cols.12.into(),
            cols.13.into(),
            cols.14.into(),
        );

        // Define the PIO program for polling GPIO pins
        let program = pio_asm!(
            ".wrap_target",
            "in pins, 32", // push the status of all GPIO pins to the RX
            ".wrap"
        );

        let installed = pio.install(&program.program).unwrap();

        // Configure polling rate and clock divisor
        const POLLING_RATE: HertzU32 = HertzU32::Hz(4000);
        let cycles_per_unit: u32 = 1; // TODO: Adjust based on actual cycles per unit
        let clk_div: (u16, u8) = {
            let bit_freq = POLLING_RATE * cycles_per_unit;

            fixed_point_div(clock_freq, bit_freq)
        };

        let (mut sm, rx, _) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
            .buffers(rp2040_hal::pio::Buffers::OnlyRx)
            .clock_divisor_fixed_point(clk_div.0, clk_div.1)
            .autopush(true)
            .push_threshold(u32::BITS as u8)
            .in_pin_base(0)
            .build(uninit_sm);

        // Set pin directions
        sm.set_pindirs([
            (concrete_rows.0.id().num, Output),
            (concrete_rows.1.id().num, Output),
            (concrete_rows.2.id().num, Output),
            (concrete_rows.3.id().num, Output),
            (concrete_rows.4.id().num, Output),
            (concrete_cols.0.id().num, Input),
            (concrete_cols.1.id().num, Input),
            (concrete_cols.2.id().num, Input),
            (concrete_cols.3.id().num, Input),
            (concrete_cols.4.id().num, Input),
            (concrete_cols.5.id().num, Input),
            (concrete_cols.6.id().num, Input),
            (concrete_cols.7.id().num, Input),
            (concrete_cols.8.id().num, Input),
            (concrete_cols.9.id().num, Input),
            (concrete_cols.10.id().num, Input),
            (concrete_cols.11.id().num, Input),
            (concrete_cols.12.id().num, Input),
            (concrete_cols.13.id().num, Input),
            (concrete_cols.14.id().num, Input),
        ]);

        // Precalculate the column mask
        let column_mask = 0x1 << concrete_cols.0.id().num
            | 0x1 << concrete_cols.1.id().num
            | 0x1 << concrete_cols.2.id().num
            | 0x1 << concrete_cols.3.id().num
            | 0x1 << concrete_cols.4.id().num
            | 0x1 << concrete_cols.5.id().num
            | 0x1 << concrete_cols.6.id().num
            | 0x1 << concrete_cols.7.id().num
            | 0x1 << concrete_cols.8.id().num
            | 0x1 << concrete_cols.9.id().num
            | 0x1 << concrete_cols.10.id().num
            | 0x1 << concrete_cols.11.id().num
            | 0x1 << concrete_cols.12.id().num
            | 0x1 << concrete_cols.13.id().num
            | 0x1 << concrete_cols.14.id().num;

        KeyboardInputManager {
            sm,
            _rows: (
                IRow1::from(concrete_rows.0),
                IRow2::from(concrete_rows.1),
                IRow3::from(concrete_rows.2),
                IRow4::from(concrete_rows.3),
                IRow5::from(concrete_rows.4),
            ),
            _cols: (
                ICol1::from(concrete_cols.0),
                ICol2::from(concrete_cols.1),
                ICol3::from(concrete_cols.2),
                ICol4::from(concrete_cols.3),
                ICol5::from(concrete_cols.4),
                ICol6::from(concrete_cols.5),
                ICol7::from(concrete_cols.6),
                ICol8::from(concrete_cols.7),
                ICol9::from(concrete_cols.8),
                ICol10::from(concrete_cols.9),
                ICol11::from(concrete_cols.10),
                ICol12::from(concrete_cols.11),
                ICol13::from(concrete_cols.12),
                ICol14::from(concrete_cols.13),
                ICol15::from(concrete_cols.14),
            ),
            column_mask
        }
    }
}
