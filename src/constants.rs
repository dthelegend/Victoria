use rp2040_hal::fugit::{HertzU32, MicrosDurationU32};

pub const NUMBER_OF_LEDS: usize = 68;
pub const RESET_DELAY: MicrosDurationU32 = MicrosDurationU32::micros((60 * NUMBER_OF_LEDS) as u32);
pub const EFFECT_RATE: HertzU32 = HertzU32::nanos(500);

//
pub const USB_ENDPOINT_POLL_RATE: HertzU32 = HertzU32::Hz(1000);
pub const KEYBOARD_POLLING_RATE: HertzU32 = HertzU32::Hz(4000);
pub const ROWS_PER_POLL: u32 = 4;
pub const HID_TICK_RATE: HertzU32 = HertzU32::millis(1);

// Keyboard specific Keymaps
const NUMBER_OF_ROWS : usize = 5;
const NUMBER_OF_COLUMNS : usize = 15;

#[rustfmt::skip]
pub mod keymaps {
    use crate::keymap::{DefaultKeymap, Keymap};
    use nalgebra::matrix;
    use usbd_human_interface_device::page::Keyboard::*;

    type DefaultKeymapT = Keymap<{super::NUMBER_OF_ROWS}, {super::NUMBER_OF_COLUMNS}>;
    pub const DEFAULT_BREAK : &[(usize,usize)] = &[(4, 0), (3, 0), (4, 1), (4, 2), (2, 9)]; // LCTRL + LSHIFT + LALT + LWIN + L 

    const BASE_KEYMAP : DefaultKeymapT = matrix![
        Escape, Keyboard1, Keyboard2, Keyboard3, Keyboard4, Keyboard5, Keyboard6, Keyboard7, Keyboard8, Keyboard9, Keyboard0, Minus, Equal, DeleteBackspace, Grave;
        Tab, Q, W, E, R, T, Y, U, I, O, P, LeftBrace, RightBrace, Backslash, Home;
        CapsLock, A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe, NoEventIndicated /*12*/, ReturnEnter, PageUp;
        LeftShift, Z, X, C, V, B, N, M, Comma, Dot, ForwardSlash, NoEventIndicated, RightShift, UpArrow, PageDown;
        LeftControl, LeftAlt, LeftGUI, NoEventIndicated /*3*/, NoEventIndicated /*4*/, Space, NoEventIndicated /*6*/, NoEventIndicated /*7*/, NoEventIndicated /*8*/, RightGUI, NoEventIndicated /* Fn */, Menu, LeftArrow, DownArrow, RightArrow;
    ];
    const DEFAULT_FUNCTION_KEYMAP : DefaultKeymapT = matrix![
        Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, DeleteForward, Grave;
        Tab, Q, W, E, R, T, Y, U, I, O, P, LeftBrace, RightBrace, Backslash, Home;
        CapsLock, A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe, NoEventIndicated /*12*/, ReturnEnter, PageUp;
        LeftShift, Z, X, C, V, B, N, M, Comma, Dot,  ForwardSlash, NoEventIndicated, RightShift, UpArrow, PageDown;
        LeftControl, LeftGUI, LeftAlt, NoEventIndicated /*3*/, NoEventIndicated /*4*/, Space, NoEventIndicated /*6*/, NoEventIndicated /*7*/, NoEventIndicated /*8*/, RightAlt, NoEventIndicated /* Fn */, Menu, LeftArrow, DownArrow, RightArrow;
    ];
    
    const DEFAULT_FN_KEY : (usize, usize) = (4, 10);
    
    pub const DEFAULT_KEYMAP : DefaultKeymap<{super::NUMBER_OF_ROWS}, {super::NUMBER_OF_COLUMNS}> = DefaultKeymap::new(
        BASE_KEYMAP,
        DEFAULT_FN_KEY,
        DEFAULT_FUNCTION_KEYMAP,
        DEFAULT_BREAK
    );
}