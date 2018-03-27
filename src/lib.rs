//! Driver for the SSD1331 colour OLED display

#![no_std]
// #![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]

extern crate embedded_hal as hal;

mod command;
pub mod interface;
pub mod builder;

// 96 x 64 pixels, 16 bits (two bytes) per pixel
// FIXME: Crashes in non-release mode, presumably consumes too much memory
type ScreenBuffer = [Pixel; 1024];

use command::*;
use interface::DisplayInterface;
use hal::blocking::delay::DelayMs;
use hal::digital::OutputPin;
pub use builder::Builder;

type Coord = (u32, u32);
type Color = u16;

type Pixel = (Coord, Color);

/// SSD1331 driver
pub struct SSD1331<DI> {
    /// SPI interface to use
    iface: DI,

    /// 96 x 64 pixel buffer, 16BPP. Pixel distribution controlled by configuration; see Table 9 in
    /// datasheet.
    buffer: ScreenBuffer,
}

impl<DI> SSD1331<DI>
where
    DI: DisplayInterface,
{
    /// Create new SSD1331 instance
    pub fn new(iface: DI) -> SSD1331<DI> {
        SSD1331 {
            iface,
            buffer: [0; 1024],
        }
    }

    /// Clear the display buffer. You need to call `disp.flush()` for any effect on the screen
    pub fn clear(&mut self) {
        // TODO
    }

    /// Reset display
    pub fn reset<RST, DELAY>(&mut self, rst: &mut RST, delay: &mut DELAY)
    where
        RST: OutputPin,
        DELAY: DelayMs<u8>,
    {
        rst.set_high();
        delay.delay_ms(1);
        rst.set_low();
        delay.delay_ms(10);
        rst.set_high();
    }

    /// Write out data to display
    pub fn flush(&mut self) -> Result<(), DI::Error> {
        let display_width = 96;
        let display_height = 64;

        Command::ColumnAddress(0, display_width - 1).send(&mut self.iface)?;
        Command::PageAddress(0.into(), (display_height - 1).into()).send(&mut self.iface)?;

        self.iface.send_data(&[1, 2, 3])
    }

    /// Turn a pixel on or off
    pub fn set_pixel(&mut self, x: u32, y: u32, value: (u8, u8, u8)) {
        let display_width: u8 = 96;

        let byte = &mut self.buffer[((y as usize) / 8 * display_width as usize) + (x as usize)];
        let bit = 1 << (y % 8);

        if value.0 == 0 {
            *byte &= !bit;
        } else {
            *byte |= bit;
        }
    }

    // Display is set up in column mode, i.e. a byte walks down a column of 8 pixels from column 0 on the left, to column _n_ on the right
    /// Initialize display in column mode.
    pub fn init(&mut self) -> Result<(), DI::Error> {
        let display_height = 64;

        Command::DisplayOn(false).send(&mut self.iface)?;
        Command::DisplayClockDiv(0x8, 0x0).send(&mut self.iface)?;
        Command::Multiplex(display_height - 1).send(&mut self.iface)?;
        Command::DisplayOffset(0).send(&mut self.iface)?;
        Command::StartLine(0).send(&mut self.iface)?;
        // TODO: Ability to turn charge pump on/off
        Command::ChargePump(true).send(&mut self.iface)?;
        Command::AddressMode(AddrMode::Horizontal).send(&mut self.iface)?;
        Command::SegmentRemap(true).send(&mut self.iface)?;
        Command::ReverseComDir(true).send(&mut self.iface)?;
        Command::Contrast(0x80, 0x80, 0x80).send(&mut self.iface)?;
        Command::PreChargePeriod(0x1, 0xF).send(&mut self.iface)?;
        Command::VcomhDeselect(VcomhLevel::Auto).send(&mut self.iface)?;
        Command::AllOn(true).send(&mut self.iface)?;
        // Command::Invert(false).send(&mut self.iface)?;
        Command::EnableScroll(false).send(&mut self.iface)?;
        Command::DisplayOn(true).send(&mut self.iface)?;

        Ok(())
    }
}
