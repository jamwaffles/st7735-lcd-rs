use num_derive::ToPrimitive;

/// ST7789 instructions.
///
/// Copied from [here](https://github.com/adafruit/Adafruit-ST7735-Library/blob/master/Adafruit_ST77xx.h)
#[derive(ToPrimitive)]
pub enum InstructionST7789 {
    NOP = 0x00,
    SWRESET = 0x01,
    RDDID = 0x04,
    RDDST = 0x09,
    SLPIN = 0x10,
    SLPOUT = 0x11,
    PTLON = 0x12,
    NORON = 0x13,
    INVOFF = 0x20,
    INVON = 0x21,
    DISPOFF = 0x28,
    DISPON = 0x29,
    CASET = 0x2A,
    RASET = 0x2B,
    RAMWR = 0x2C,
    RAMRD = 0x2E,
    PTLAR = 0x30,
    COLMOD = 0x3A,
    MADCTL = 0x36,
    MADCTL_MY = 0x80,
    MADCTL_MX = 0x40,
    // MADCTL_MV = 0x20,
    // MADCTL_ML = 0x10,
    // MADCTL_RGB = 0x00,
    RDID1 = 0xDA,
    RDID2 = 0xDB,
    RDID3 = 0xDC,
    RDID4 = 0xDD,
}
