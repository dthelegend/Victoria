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

    pub fn get_report(&mut self) -> impl Iterator<Item = Keyboard>{
        gen {
            macro_rules! scan_rows {
                ( $col_idx:tt , [ $( $row_idx:tt ),+ ] ) => {
                    $(if self.rows.$row_idx.is_high().unwrap() {
                        yield self.keymap.get_key($col_idx, $row_idx);
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
}

pub trait KeyMap {
    fn get_key(&mut self, col: u8, row: u8) -> Keyboard;
}

pub struct BasicKeymap();

impl KeyMap for BasicKeymap {
    fn get_key(&mut self, col: u8, row: u8) -> Keyboard {
        match (col, row) {
            (0, 0) => panic!(),
            _ => Keyboard::A
        }
    }
}
