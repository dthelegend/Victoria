use embedded_hal::digital::{InputPin, OutputPin};
use nalgebra::SMatrix;
use rp2040_hal::gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown, PullUp};


pub type ActivationMatrix<const NROW: usize, const NCOL: usize> = SMatrix<bool, NROW, NCOL>;

pub struct KeyboardInputManager<const NRows: usize, const NCols: usize> {
    rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NRows],
    cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCols],
}

impl <const NRows: usize, const NCols: usize> KeyboardInputManager<NRows, NCols> {
    pub const fn from_pins(
        rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NRows],
        cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCols]) -> Self {
        KeyboardInputManager { rows, cols }
    }

    pub fn activate(self) -> ActiveKeyboardManager<NRows, NCols> {
        let Self { rows, cols } = self;

        ActiveKeyboardManager::create(rows, cols)
    }
}

pub struct ActiveKeyboardManager<const NRows: usize, const NCols: usize>
{
    // const-ish vars
    rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NRows],
    cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCols],

    // mut vars
    key_buffer: ActivationMatrix<NRows, NCols>,
    col_number: usize,
}

impl<const NRows: usize, const NCols: usize>
    ActiveKeyboardManager<NRows, NCols>
{
    fn create(
        rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NRows],
        mut cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCols],
    ) -> Self {
        cols[0].set_high().unwrap();

        Self {
            rows,
            cols,

            key_buffer: ActivationMatrix::from_element(false),
            col_number: 0,
        }
    }

    pub fn continue_polling(&mut self) -> Option<&ActivationMatrix<NRows, NCols>> {
        for (i, row_pin) in self.rows.iter_mut().enumerate() {
            self.key_buffer[(i, self.col_number)] = row_pin.is_high().unwrap();
        }

        self.cols[self.col_number].set_low().unwrap();

        let mut output = None;
        // Ensure col number invariant
        self.col_number += 1;
        if self.col_number >= NCols {
            self.col_number = 0;
            output = Some(&self.key_buffer)
        }

        self.cols[self.col_number].set_high().unwrap();

        output
    }
}

