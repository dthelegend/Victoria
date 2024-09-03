pub use rp2040_hal as hal;

pub use hal::entry;

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

hal::bsp_pins!(
    // Columns
    Gpio22 {
        name: col1,
        aliases: { FunctionSioOutput, PullUp: Col1 }
    },
    Gpio11 {
        name: col2,
        aliases: { FunctionSioOutput, PullUp: Col2 }
    },
    Gpio10 {
        name: col3,
        aliases: { FunctionSioOutput, PullUp: Col3 }
    },
    Gpio9 {
        name: col4,
        aliases: { FunctionSioOutput, PullUp: Col4 }
    },
    Gpio8 {
        name: col5,
        aliases: { FunctionSioOutput, PullUp: Col5 }
    },
    Gpio7 {
        name: col6,
        aliases: { FunctionSioOutput, PullUp: Col6 }
    },
    Gpio6 {
        name: col7,
        aliases: { FunctionSioOutput, PullUp: Col7 }
    },
    Gpio5 {
        name: col8,
        aliases: { FunctionSioOutput, PullUp: Col8 }
    },
    Gpio4 {
        name: col9,
        aliases: { FunctionSioOutput, PullUp: Col9 }
    },
    Gpio3 {
        name: col10,
        aliases: { FunctionSioOutput, PullUp: Col10 }
    },
    Gpio2 {
        name: col11,
        aliases: { FunctionSioOutput, PullUp: Col11 }
    },
    Gpio1 {
        name: col12,
        aliases: { FunctionSioOutput, PullUp: Col12 }
    },
    Gpio0 {
        name: col13,
        aliases: { FunctionSioOutput, PullUp: Col13 }
    },
    Gpio13 {
        name: col14,
        aliases: { FunctionSioOutput, PullUp: Col14 }
    },
    Gpio12 {
        name: col15,
        aliases: { FunctionSioOutput, PullUp: Col15 }
    },

    // Rows
    Gpio24 {
        name: row1,
        aliases: { FunctionSioInput, PullUp: Row1 }
    },
    Gpio27 {
        name: row2,
        aliases: { FunctionSioInput, PullUp: Row2 }
    },
    Gpio23 {
        name: row3,
        aliases: { FunctionSioInput, PullUp: Row3 }
    },
    Gpio14 {
        name: row4,
        aliases: { FunctionSioInput, PullUp: Row4 }
    },
    Gpio15 {
        name: row5,
        aliases: { FunctionSioInput, PullUp: Row5 }
    },

    // RGB LEDs
    Gpio25 {
        name: rgb_data,
        aliases: { FunctionPio0, PullUp: RGBData }
    },
    Gpio26 {
        name: rgb_enable,
        aliases: { FunctionSioOutput, PullUp: RGBEnable }
    },
);

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
