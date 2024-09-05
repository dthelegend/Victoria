use crate::hal::{
    Col1, Col10, Col11, Col12, Col13, Col14, Col15, Col2, Col3, Col4, Col5, Col6, Col7, Col8, Col9,
    Row1, Row2, Row3, Row4, Row5,
};
use embedded_hal::digital::{InputPin, OutputPin};

use usbd_human_interface_device::page::Keyboard;

// TODO improve this whole file

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

pub struct KeyboardInputManager {
    rows: RowsPinGroup,
    cols: ColsPinGroup,
}

impl KeyboardInputManager {
    pub fn initialise(rows: RowsPinGroup, cols: ColsPinGroup) -> Self {
        KeyboardInputManager { rows, cols }
    }

    pub fn activate_with_keymap<KM: KeyMap>(self, keymap: KM) -> ActiveKeyboardManager<KM> {
        let Self { rows, mut cols } = self;

        cols.0.set_high().unwrap();

        ActiveKeyboardManager {
            rows,
            cols,
            keymap,
            column_idx: 0,
            key_buffer: [Keyboard::NoEventIndicated; 15 * 5],
        }
    }
}

pub struct ActiveKeyboardManager<KM: KeyMap> {
    // const-ish vars
    rows: RowsPinGroup,
    cols: ColsPinGroup,
    keymap: KM,

    // mut vars
    key_buffer: [Keyboard; 15 * 5],
    column_idx: u8, // invariant: this must be < 5
}

impl<KM: KeyMap> ActiveKeyboardManager<KM> {
    pub fn continue_polling(&mut self) -> Option<[Keyboard; 15 * 5]> {
        macro_rules! get_rows {
            ( $($i:tt ),+ ) => {
                {
                    $(self.key_buffer[self.column_idx as usize * 5 + $i] = {
                        if (self.rows.$i.is_high().unwrap()) {
                            self.keymap.get_key($i, self.column_idx)
                        } else {
                            Keyboard::NoEventIndicated
                        }
                    };)+
                }
            };
        }

        macro_rules! match_apply_to_col {
            ( $to_match:expr, $to_apply_on_match:path, [ $($over:tt),+ ] ) => {
                match $to_match {
                    $($over => $to_apply_on_match(&mut self.cols.$over),)+
                    _ => unreachable!()
                }
            };
        }

        get_rows!(0, 1, 2, 3, 4);

        match_apply_to_col!(
            self.column_idx,
            OutputPin::set_low,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        )
        .unwrap();

        self.column_idx += 1;

        let output = if self.column_idx > 14 {
            self.column_idx = 0;

            Some(self.key_buffer)
        } else {
            None
        };

        match_apply_to_col!(
            self.column_idx,
            OutputPin::set_high,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        )
        .unwrap();

        output
    }
}

pub trait KeyMap {
    fn get_key(&mut self, row: u8, col: u8) -> Keyboard;
}

macro_rules! declare_keymaps {
    { $(
        $vv:vis struct $name:ident {
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
            impl KeyMap for $name {
                fn get_key(&mut self, row: u8, col: u8) -> Keyboard {
                    match (row, col) {
                        $(
                            $(( $row, $col ) => $out,)*
                        )*
                        _ => unreachable!()
                    }
                }
            }
        )+
    }
}

declare_keymaps! {
    pub struct BasicKeymap {
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
            10 => panic!(), // Function key acts as a soft reset for the meantime
            11 => Keyboard::M,
            12 => Keyboard::LeftArrow,
            13 => Keyboard::DownArrow,
            14 => Keyboard::RightArrow,
        }
    }
}
