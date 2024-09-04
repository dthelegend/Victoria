use embedded_hal::digital::{InputPin, OutputPin};

use crate::hal::{
    Col1, Col10, Col11, Col12, Col13, Col14, Col15, Col2, Col3, Col4, Col5, Col6, Col7, Col8, Col9,
    Row1, Row2, Row3, Row4, Row5,
};

use usbd_human_interface_device::page::Keyboard;

type RowsTuple = (Row1, Row2, Row3, Row4, Row5);
type ColsTuple = (
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
    rows: RowsTuple,
    cols: ColsTuple,
}

impl KeyboardInputManager {
    pub fn initialise(
        rows: RowsTuple,
        cols: ColsTuple,
    ) -> Self {
        KeyboardInputManager { rows, cols }
    }

    pub fn activate_with_keymap<KM : KeyMap>(self, keymap: KM) -> ActiveKeyboardManager<KM> {
        let Self { rows, cols } = self;

        ActiveKeyboardManager {
            rows,
            cols,
            keymap
        }
    }
}

pub struct ActiveKeyboardManager<KM : KeyMap> {
    rows: RowsTuple,
    cols: ColsTuple,
    keymap: KM
}

impl <KM : KeyMap> ActiveKeyboardManager<KM> {

    pub fn get_report(&mut self) -> [Keyboard; 15 * 5] {
        let mut output = [Keyboard::NoEventIndicated; 15 * 5];
        
        // MEFINAE
        macro_rules! scan_rows {
            ( $col_idx:tt , [ $( $row_idx:tt ),+ ] ) => {
                $(if self.rows.$row_idx.is_high().unwrap() {
                     output[$col_idx * 5 + $row_idx] = self.keymap.get_key($col_idx, $row_idx);
                })+
            };
        }

        macro_rules! scan_cols {
            ( $( $col_idx:tt ),+ ) => {
                $({
                    self.cols.$col_idx.set_high().unwrap();

                    scan_rows!($col_idx, [ 0, 1, 2, 3, 4 ]);

                    self.cols.$col_idx.set_low().unwrap();
                })+
            };
        }

        scan_cols!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
        
        output

        // Example:
        // {
        //     self.cols.0.set_high().unwrap();
        //
        //     if self.rows.0.is_high().unwrap() {
        //         yield self.keymap.get_key(0, 0);
        //     }
        //     if self.rows.1.is_high().unwrap() {
        //         yield self.keymap.get_key(0, 1);
        //     }
        //     if self.rows.2.is_high().unwrap() {
        //         yield self.keymap.get_key(0, 2);
        //     }
        //     if self.rows.3.is_high().unwrap() {
        //         yield self.keymap.get_key(0, 3);
        //     }
        //     if self.rows.4.is_high().unwrap() {
        //         yield self.keymap.get_key(0, 4);
        //     }
        //
        //     self.cols.0.set_low().unwrap();
        // }
    }
}

pub trait KeyMap {
    fn get_key(&mut self, col: u8, row: u8) -> Keyboard;
}

pub struct BasicKeymap();

impl KeyMap for BasicKeymap {
    fn get_key(&mut self, col: u8, row: u8) -> Keyboard {
        match (row, col) {
            // (0, 0) => Keyboard::Grave,
            // (0, 1) => Keyboard::Keyboard1,
            // (0, 2) => Keyboard::Keyboard2,
            // (0, 3) => Keyboard::Keyboard3,
            // (0, 4) => Keyboard::Keyboard4,
            // (0, 5) => Keyboard::Keyboard5,
            // (0, 6) => Keyboard::Keyboard6,
            // (0, 7) => Keyboard::Keyboard7,
            // (0, 8) => Keyboard::Keyboard8,
            // (0, 9) => Keyboard::Keyboard9,
            // (0, 10) => Keyboard::Keyboard0,
            // (0, 11) => Keyboard::Minus,
            // (0, 12) => Keyboard::Equal,
            // (0, 13) => Keyboard::DeleteBackspace,
            // (0, 14) => Keyboard::Escape,
            (_, 0) => Keyboard::A,
            (_, 1) => Keyboard::B,
            (_, 2) => Keyboard::C,
            (_, 3) => Keyboard::D,
            (_, 4) => Keyboard::E,
            (_, 5) => Keyboard::F,
            (_, 6) => Keyboard::G,
            (_, 7) => Keyboard::H,
            (_, 8) => Keyboard::I,
            (_, 9) => Keyboard::J,
            (_, 10) => Keyboard::K,
            (_, 11) => Keyboard::L,
            (_, 12) => Keyboard::M,
            (_, 13) => Keyboard::N,
            (_, 14) => panic!(),
            // _ => Keyboard::NoEventIndicated
            _ => unreachable!()
        }
    }
}
