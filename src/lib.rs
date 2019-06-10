#![no_std]

//! This crate provides a ST7735 driver to connect to TFT displays.

pub mod instruction;

extern crate stm32f1xx_hal as hal;

use core::mem::transmute;

use crate::instruction::Instruction;
use embedded_hal::blocking::spi;
use embedded_hal::digital::OutputPin;
use embedded_hal::timer::{CountDown, Periodic};
use hal::delay::Delay;
use hal::prelude::*;
use nb::block;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

/// ST7735 driver to connect to TFT displays.
pub struct ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// SPI
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Reset pin.
    rst: RST,

    /// Delay
    timer: Delay,

    /// Whether the display is RGB (true) or BGR (false)
    rgb: bool,

    /// Whether the colours are inverted (true) or not (false)
    inverted: bool,
}

/// Display orientation.
#[derive(ToPrimitive)]
pub enum Orientation {
    Portrait = 0x00,
    Landscape = 0x60,
    PortraitSwapped = 0xC0,
    LandscapeSwapped = 0xA0,
}

impl<SPI, DC, RST> ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(spi: SPI, dc: DC, rst: RST, timer: Delay, rgb: bool, inverted: bool) -> Self
    where
        SPI: spi::Write<u8>,
        DC: OutputPin,
        RST: OutputPin,
    {
        let display = ST7735 {
            spi,
            dc,
            rst,
            timer,
            rgb,
            inverted,
        };

        display
    }

    /// Runs commands to initialize the display.
    pub fn init(&mut self) -> Result<(), ()> {
        self.hard_reset();
        self.write_command(Instruction::SWRESET, None)?;
        // block!(self.timer.wait()).map_err(|_|())?;
        self.timer.delay_ms(50u32);
        self.write_command(Instruction::SLPOUT, None)?;
        self.timer.delay_ms(50u32);
        // block!(self.timer.wait()).map_err(|_|())?;
        self.write_command(Instruction::FRMCTR1, Some(&[0x01, 0x2C, 0x2D]))?;
        self.write_command(Instruction::FRMCTR2, Some(&[0x01, 0x2C, 0x2D]))?;
        self.write_command(
            Instruction::FRMCTR3,
            Some(&[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D]),
        )?;
        self.write_command(Instruction::INVCTR, Some(&[0x07]))?;
        self.write_command(Instruction::PWCTR1, Some(&[0xA2, 0x02, 0x84]))?;
        self.write_command(Instruction::PWCTR2, Some(&[0xC5]))?;
        self.write_command(Instruction::PWCTR3, Some(&[0x0A, 0x00]))?;
        self.write_command(Instruction::PWCTR4, Some(&[0x8A, 0x2A]))?;
        self.write_command(Instruction::PWCTR5, Some(&[0x8A, 0xEE]))?;
        self.write_command(Instruction::VMCTR1, Some(&[0x0E]))?;
        if self.inverted {
            self.write_command(Instruction::INVON, None)?;
        } else {
            self.write_command(Instruction::INVOFF, None)?;
        }
        if self.rgb {
            self.write_command(Instruction::MADCTL, Some(&[0x00]))?;
        } else {
            self.write_command(Instruction::MADCTL, Some(&[0x08]))?;
        }
        self.write_command(Instruction::COLMOD, Some(&[0x05]))?;
        self.write_command(Instruction::DISPON, None)?;
        self.timer.delay_ms(50u32);
        // block!(self.timer.wait()).map_err(|_| ())?;
        Ok(())
    }

    pub fn hard_reset(&mut self) {
        self.rst.set_high();
        self.rst.set_low();
        self.rst.set_high();
    }

    fn write_command(&mut self, command: Instruction, params: Option<&[u8]>) -> Result<(), ()> {
        self.dc.set_low();
        self.spi
            .write(&[command.to_u8().unwrap()])
            .map_err(|_| ())?;
        if params.is_some() {
            self.write_data(params.unwrap())?;
        }
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.dc.set_high();
        self.spi.write(data).map_err(|_| ())?;
        Ok(())
    }

    /// Writes a data word to the display.
    fn write_word(&mut self, value: u16) -> Result<(), ()> {
        let bytes: [u8; 2] = unsafe { transmute(value.to_be()) };
        self.write_data(&bytes)?;
        Ok(())
    }

    pub fn set_orientation(&mut self, orientation: &Orientation) -> Result<(), ()> {
        if self.rgb {
            self.write_command(Instruction::MADCTL, Some(&[orientation.to_u8().unwrap()]))?;
        } else {
            self.write_command(
                Instruction::MADCTL,
                Some(&[orientation.to_u8().unwrap() | 0x08]),
            )?;
        }
        Ok(())
    }

    /// Sets the address window for the display.
    fn set_address_window(&mut self, x: u16, y: u16, w: u16, h: u16) -> Result<(), ()> {
        self.write_command(Instruction::CASET, None)?;
        self.write_word(x)?;
        self.write_word(w)?;
        self.write_command(Instruction::RASET, None)?;
        self.write_word(y)?;
        self.write_word(h)?;
        Ok(())
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), ()> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RAMWR, None)?;
        self.write_word(color)?;
        Ok(())
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::{drawable, pixelcolor::PixelColorU16, Drawing};

#[cfg(feature = "graphics")]
impl<SPI, DC, RST> Drawing<PixelColorU16> for ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = drawable::Pixel<PixelColorU16>>,
    {
        for pixel in item_pixels {
            self.set_pixel((pixel.0).0 as u16, (pixel.0).1 as u16, pixel.1.into_inner())
                .expect("pixel write failed");
        }
    }
}
