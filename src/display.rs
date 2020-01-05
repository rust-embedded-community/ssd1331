use hal::blocking::delay::DelayMs;
use hal::digital::v2::OutputPin;

use crate::displayrotation::DisplayRotation;
use crate::error::Error;
use crate::properties::Properties;
use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// 96px x 64px screen with 16 bits (2 bytes) per pixel
const BUF_SIZE: usize = 12288;

/// SSD1331 display interface
///
/// This is the main display driver interface. It is recommended that the [`Builder`] is used to
/// create instances of it.
///
/// # Examples
///
/// ## Draw shapes and text with [`embedded-graphics`]
///
/// This requires the `graphics` feature to be enabled
///
/// ```rust
/// use ssd1331::Builder;
/// use embedded_graphics::{
///     prelude::*,
///     fonts::Font6x8,
///     geometry::Point,
///     image::ImageLE,
///     pixelcolor::Rgb565,
///     primitives::{Circle, Line, Rectangle},
///     Drawing,
/// };
/// # use ssd1331::test_helpers::{Pin, Spi};
///
/// // Set up SPI interface and digital pin. These are stub implementations used in examples.
/// let spi = Spi;
/// let dc = Pin;
///
/// let mut display = Builder::new().connect_spi(spi, dc);
/// let image = ImageLE::new(include_bytes!("../examples/ferris.raw"), 86, 64);
///
/// // Initialise and clear the display
/// display.init().unwrap();
/// display.flush().unwrap();
///
/// display.draw(
///     Line::new(Point::new(0, 0), Point::new(16, 16))
///         .stroke(Some(Rgb565::RED))
///         .stroke_width(1)
///         .into_iter(),
/// );
/// display.draw(
///     Rectangle::new(Point::new(24, 0), Point::new(40, 16))
///         .stroke(Some(Rgb565::new(255, 127, 0)))
///         .stroke_width(1)
///         .into_iter(),
/// );
/// display.draw(
///     Circle::new(Point::new(64, 8), 8)
///         .stroke(Some(Rgb565::GREEN))
///         .stroke_width(1)
///         .into_iter(),
/// );
/// display.draw(&image);
/// display.draw(
///     Font6x8::render_str("Hello Rust!")
///         .translate(Point::new(24, 24))
///         .style(Style::stroke(Rgb565::RED))
///         .into_iter(),
/// );
///
/// // Render graphics objects to the screen
/// display.flush().unwrap();
/// ```
///
/// [`embedded-graphics`]: https://crates.io/crates/embedded-graphics
/// [`Builder`]: ./struct.Builder.html
pub struct Ssd1331<SPI, DC> {
    properties: Properties<SPI, DC>,
    buffer: [u8; BUF_SIZE],
}

impl<SPI, DC, CommE, PinE> Ssd1331<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8, Error = CommE>,
    DC: OutputPin<Error = PinE>,
{
    /// Create new GraphicsMode instance
    ///
    /// Allocates a buffer of 96px * 64px * 16bits = 12,288 bytes. This may be too large for your
    /// hardware, so check your datasheet!
    pub fn new(properties: Properties<SPI, DC>) -> Self {
        Self {
            properties,
            buffer: [0; BUF_SIZE],
        }
    }

    /// Release all resources used by GraphicsMode
    pub fn release(self) -> Properties<SPI, DC> {
        self.properties
    }

    /// Clear the display buffer. You need to call `disp.flush()` for any effect on the screen
    pub fn clear(&mut self) {
        self.buffer = [0; BUF_SIZE];
    }

    /// Reset display
    pub fn reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), Error<CommE, PinE>>
    where
        RST: OutputPin<Error = PinE>,
        DELAY: DelayMs<u8>,
    {
        rst.set_high().map_err(Error::Pin)?;
        delay.delay_ms(1);
        rst.set_low().map_err(Error::Pin)?;
        delay.delay_ms(10);
        rst.set_high().map_err(Error::Pin)?;

        Ok(())
    }

    /// Write out data to display
    pub fn flush(&mut self) -> Result<(), Error<CommE, PinE>> {
        // Ensure the display buffer is at the origin of the display before we send the full frame
        // to prevent accidental offsets
        self.properties
            .set_draw_area((0, 0), (DISPLAY_WIDTH, DISPLAY_HEIGHT))?;

        self.properties.draw(&self.buffer)
    }

    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let display_rotation = self.properties.rotation();

        let idx = match display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                if x >= DISPLAY_WIDTH as u32 {
                    return;
                }
                ((y as usize) * DISPLAY_WIDTH as usize) + (x as usize)
            }

            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                if y >= DISPLAY_WIDTH as u32 {
                    return;
                }
                ((y as usize) * DISPLAY_HEIGHT as usize) + (x as usize)
            }
        } * 2;

        if idx >= self.buffer.len() - 1 {
            return;
        }

        // Split 16 bit value into two bytes
        let low = (value & 0xff) as u8;
        let high = ((value & 0xff00) >> 8) as u8;

        self.buffer[idx] = high;
        self.buffer[idx + 1] = low;
    }

    /// Display is set up in column mode, i.e. a byte walks down a column of 8 pixels from
    /// column 0 on the left, to column _n_ on the right
    pub fn init(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.properties.init_column_mode()?;
        Ok(())
    }

    /// Get display dimensions, taking into account the current rotation of the display
    pub fn dimensions(&self) -> (u8, u8) {
        self.properties.dimensions()
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), Error<CommE, PinE>> {
        self.properties.set_rotation(rot)
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics::{
    drawable,
    pixelcolor::{
        raw::{RawData, RawU16},
        Rgb565,
    },
    Drawing,
};

#[cfg(feature = "graphics")]
impl<SPI, DC> Drawing<Rgb565> for Ssd1331<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = drawable::Pixel<Rgb565>>,
    {
        // Filter out pixels that are off the top left of the screen
        let on_screen_pixels = item_pixels
            .into_iter()
            .filter(|drawable::Pixel(point, _)| point.x >= 0 && point.y >= 0);

        for drawable::Pixel(point, color) in on_screen_pixels {
            // NOTE: The filter above means the coordinate conversions from `i32` to `u32` should
            // never error.
            self.set_pixel(
                point.x as u32,
                point.y as u32,
                RawU16::from(color).into_inner(),
            );
        }
    }
}
