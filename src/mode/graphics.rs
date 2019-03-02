//! Buffered display module for use with the [embedded_graphics] crate
//!
//! ```rust,ignore
//! let spi = /* SPI interface from your HAL of choice */;
//! let dc = /* DC pin from your HAL of choice */;
//! let display: GraphicsMode<_> = Builder::new().connect_spi(dc, spi).into();
//! let image = include_bytes!("image_16x16.raw");
//!
//! display.init().unwrap();
//! display.flush().unwrap();
//! display.draw(Line::new(Coord::new(0, 0), (16, 16), 1.into()).into_iter());
//! display.draw(Rect::new(Coord::new(24, 0), (40, 16), 1u8.into()).into_iter());
//! display.draw(Circle::new(Coord::new(64, 8), 8, 1u8.into()).into_iter());
//! display.draw(Image1BPP::new(image, 0, 24));
//! display.draw(Font6x8::render_str("Hello Rust!", 1u8.into()).translate(Coord::new(24, 24)).into_iter());
//! display.flush().unwrap();
//! ```

use hal::blocking::delay::DelayMs;
use hal::digital::OutputPin;

use crate::displayrotation::DisplayRotation;
use crate::interface::DisplayInterface;
use crate::mode::displaymode::DisplayModeTrait;
use crate::properties::DisplayProperties;
use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// 96px x 64px screen with 16 bits (2 bytes) per pixel
const BUF_SIZE: usize = 12288;

/// Graphics mode handler
pub struct GraphicsMode<DI>
where
    DI: DisplayInterface,
{
    properties: DisplayProperties<DI>,
    buffer: [u8; BUF_SIZE],
}

impl<DI> DisplayModeTrait<DI> for GraphicsMode<DI>
where
    DI: DisplayInterface,
{
    /// Create new GraphicsMode instance
    ///
    /// Allocates a buffer of 96px * 64px * 16bits = 12,288 bytes. This may be too large for your
    /// hardware, so check your datasheet!
    fn new(properties: DisplayProperties<DI>) -> Self {
        GraphicsMode {
            properties,
            buffer: [0; BUF_SIZE],
        }
    }

    /// Release all resources used by GraphicsMode
    fn release(self) -> DisplayProperties<DI> {
        self.properties
    }
}

impl<DI> GraphicsMode<DI>
where
    DI: DisplayInterface,
{
    /// Clear the display buffer. You need to call `disp.flush()` for any effect on the screen
    pub fn clear(&mut self) {
        self.buffer = [0; BUF_SIZE];
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
    pub fn flush(&mut self) -> Result<(), ()> {
        // Ensure the display buffer is at the origin of the display before we send the full frame
        // to prevent accidental offsets
        self.properties
            .set_draw_area((0, 0), (DISPLAY_WIDTH, DISPLAY_HEIGHT))?;

        self.properties.draw(&self.buffer)
    }

    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let display_rotation = self.properties.get_rotation();

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
    pub fn init(&mut self) -> Result<(), ()> {
        self.properties.init_column_mode()?;
        Ok(())
    }

    /// Get display dimensions, taking into account the current rotation of the display
    pub fn get_dimensions(&self) -> (u8, u8) {
        self.properties.get_dimensions()
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), ()> {
        self.properties.set_rotation(rot)
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::{drawable, pixelcolor::PixelColorU16, Drawing};

#[cfg(feature = "graphics")]
impl<DI> Drawing<PixelColorU16> for GraphicsMode<DI>
where
    DI: DisplayInterface,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = drawable::Pixel<PixelColorU16>>,
    {
        for pixel in item_pixels {
            self.set_pixel((pixel.0).0, (pixel.0).1, pixel.1.into_inner());
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO lol
}
