use crate::common::{Assert, IsTrue};
use crate::hal::{
    Col1, Col10, Col11, Col12, Col13, Col14, Col15, Col2, Col3, Col4, Col5, Col6, Col7, Col8, Col9,
    Row1, Row2, Row3, Row4, Row5,
};
use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown, PullUp};
use usbd_human_interface_device::page::Keyboard;

type RowsPinGroup = (Row1, Row2, Row3, Row4, Row5);

type ColsPinGroup = (
    Col1,
    Col2,
    Col3,
    Col4,
    Col5,
    Col6,
    Col7,
    Col8,
    Col9,
    Col10,
    Col11,
    Col12,
    Col13,
    Col14,
    Col15,
);

macro_rules! tuple_to_dyn {
    ( $t:expr, [ $($i:tt),* ] ) => {
        [
            $($t.$i.into_dyn_pin(),)*
        ]
    };
}

pub struct KeyboardInputManager {
    rows: RowsPinGroup,
    cols: ColsPinGroup,
}

impl KeyboardInputManager {
    pub fn initialise(rows: RowsPinGroup, cols: ColsPinGroup) -> Self {
        KeyboardInputManager { rows, cols }
    }

    pub fn activate(self) -> ActiveKeyboardManager<5, 15, { 5 * 15 }> {
        let Self { mut rows, mut cols } = self;

        cols.0.set_high().unwrap();

        ActiveKeyboardManager::create(
            tuple_to_dyn!(rows, [0, 1, 2, 3, 4]),
            tuple_to_dyn!(cols, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]),
        )
    }
}

pub struct ActiveKeyboardManager<const NROW: usize, const NCOL: usize, const NKeys: usize>
where
    Assert<{ NCOL * NROW == NKeys }>: IsTrue,
{
    // const-ish vars
    rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NROW],
    cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCOL],

    // mut vars
    key_buffer: [bool; NKeys],
    col_number: usize,
}

impl<const NRow: usize, const NCOL: usize, const NKEYS: usize>
    ActiveKeyboardManager<NRow, NCOL, NKEYS>
where
    Assert<{ NCOL * NRow == NKEYS }>: IsTrue,
{
    fn create(
        rows: [Pin<DynPinId, FunctionSioInput, PullDown>; NRow],
        mut cols: [Pin<DynPinId, FunctionSioOutput, PullUp>; NCOL],
    ) -> Self {
        cols[0].set_high().unwrap();

        Self {
            rows,
            cols,

            key_buffer: [false; NKEYS],
            col_number: 0,
        }
    }

    pub fn continue_polling(&mut self) -> Option<[bool; NKEYS]> {
        for (i, row_pin) in self.rows.iter_mut().enumerate() {
            self.key_buffer[NRow * self.col_number + i] = row_pin.is_high().unwrap();
        }

        self.cols[self.col_number].set_low().unwrap();

        let mut output = None;
        // Ensure col number invariant
        self.col_number += 1;
        if self.col_number >= NCOL {
            self.col_number = 0;
            output = Some(self.key_buffer)
        }

        self.cols[self.col_number].set_high().unwrap();

        output
    }
}

// TODO Add abstractions for function layers
pub trait KeyMap<const NRow: usize, const NCol: usize, const NKeys: usize>
where
    Assert<{ NCol * NRow == NKeys }>: IsTrue,
{
    fn transform(input_buffer: [bool; NKeys]) -> impl Iterator<Item = Keyboard>;
}

macro_rules! declare_keymaps {
    { $(
        $vv:vis struct $name:ident<$nrows:tt, $ncols:tt> {
            $(
                $row:pat_param => {
                    $(
                        $col:pat => $out:expr
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ),+ } => {
        $(
            $vv struct $name ();
            impl $name {
                const INTERNAL_MAP: [Keyboard; { 5 * 15 }] = const {
                    let mut output = [Keyboard::NoEventIndicated; { 5 * 15 }];
                    let mut i = 0;

                    while i < $ncols {
                        let mut j = 0;

                        while j < $nrows {
                            output[i * $nrows + j] =  match (j, i) {
                                $(
                                    $(( $row, $col ) => $out,)*
                                )*
                                _ => Keyboard::NoEventIndicated
                            };

                            j += 1;
                        }

                        i += 1;
                    }

                    output
                };
            }
            impl KeyMap<$nrows, $ncols, {$nrows * $ncols}> for $name {
                fn transform(input_buffer: [bool; {$nrows * $ncols}]) -> impl Iterator<Item = Keyboard> {
                    Self::INTERNAL_MAP.iter().cloned().zip(input_buffer).map(|(x, y)| if y { x } else { Keyboard::NoEventIndicated })
                }
            }
        )+
    }
}

declare_keymaps! {
    pub struct BasicKeymap<5, 15> {
        0 => {
            0 => Keyboard::Grave,
            1 => Keyboard::Keyboard1,
            2 => Keyboard::Keyboard2,
            3 => Keyboard::Keyboard3,
            4 => Keyboard::Keyboard4,
            5 => Keyboard::Keyboard5,
            6 => Keyboard::Keyboard6,
            7 => Keyboard::Keyboard7,
            8 => Keyboard::Keyboard8,
            9 => Keyboard::Keyboard9,
            10 => Keyboard::Keyboard0,
            11 => Keyboard::Minus,
            12 => Keyboard::Equal,
            13 => Keyboard::DeleteBackspace,
            14 => Keyboard::Escape,
        },
        1 => {
            0 => Keyboard::Tab,
            1 => Keyboard::Q,
            2 => Keyboard::W,
            3 => Keyboard::E,
            4 => Keyboard::R,
            5 => Keyboard::T,
            6 => Keyboard::Y,
            7 => Keyboard::U,
            8 => Keyboard::I,
            9 => Keyboard::O,
            10 => Keyboard::P,
            11 => Keyboard::LeftBrace,
            12 => Keyboard::RightBrace,
            13 => Keyboard::Backslash,
            14 => Keyboard::Home,
        },
        2 => {
            0 => Keyboard::CapsLock,
            1 => Keyboard::A,
            2 => Keyboard::S,
            3 => Keyboard::D,
            4 => Keyboard::F,
            5 => Keyboard::G,
            6 => Keyboard::H,
            7 => Keyboard::J,
            8 => Keyboard::K,
            9 => Keyboard::L,
            10 => Keyboard::Semicolon,
            11 => Keyboard::Apostrophe,
            // No key 12
            13 => Keyboard::ReturnEnter,
            14 => Keyboard::PageUp,
        },
        3 => {
            0 => Keyboard::LeftShift,
            1 => Keyboard::Z,
            2 => Keyboard::X,
            3 => Keyboard::C,
            4 => Keyboard::V,
            5 => Keyboard::B,
            6 => Keyboard::N,
            7 => Keyboard::M,
            8 => Keyboard::Comma,
            9 => Keyboard::Dot,
            10 => Keyboard::ForwardSlash,
            12 => Keyboard::RightShift,
            13 => Keyboard::UpArrow,
            14 => Keyboard::PageDown,
        },
        4 => {
            0 => Keyboard::LeftControl,
            1 => Keyboard::LeftGUI,
            2 => Keyboard::LeftAlt,
            // No keys 3..=4
            5 => Keyboard::Space,
            // No keys 6..=8
            9 => Keyboard::RightAlt,
            // 10 => Function key
            11 => Keyboard::Menu,
            12 => Keyboard::LeftArrow,
            13 => Keyboard::DownArrow,
            14 => Keyboard::RightArrow,
        }
    }
}
