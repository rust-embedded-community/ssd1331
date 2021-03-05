use hal::{blocking::delay::DelayMs, digital::v2::OutputPin};

use crate::{
    command::{AddressIncrementMode, ColorMode, Command, VcomhLevel},
    displayrotation::DisplayRotation,
    error::Error,
    DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

/// 96px x 64px screen with 16 bits (2 bytes) per pixel
const BUF_SIZE: usize = 96 * 64 * 2;

/// SSD1331 display interface
///
/// # Examples
///
/// ## Draw shapes and text with [`embedded-graphics`]
///
/// This requires the `graphics` feature to be enabled (on by default).
///
/// ```rust
/// use embedded_graphics::{
///     fonts::{Font6x8, Text},
///     geometry::Point,
///     image::{Image, ImageRawLE},
///     pixelcolor::Rgb565,
///     prelude::*,
///     primitives::{Circle, Line, Rectangle},
///     style::{PrimitiveStyleBuilder, TextStyleBuilder},
/// };
/// use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
/// # use ssd1331::test_helpers::{Pin, Spi};
///
/// // Set up SPI interface and digital pin. These are stub implementations used in examples.
/// let spi = Spi;
/// let dc = Pin;
///
/// let mut display = Ssd1331::new(spi, dc, Rotate0);
/// let raw = ImageRawLE::new(include_bytes!("../examples/ferris.raw"), 86, 64);
///
/// let image: Image<ImageRawLE<Rgb565>, Rgb565> = Image::new(&raw, Point::zero());
///
/// // Initialise and clear the display
/// display.init().unwrap();
/// display.flush().unwrap();
///
/// Line::new(Point::new(0, 0), Point::new(16, 16))
///     .into_styled(
///         PrimitiveStyleBuilder::new()
///             .stroke_color(Rgb565::RED)
///             .stroke_width(1)
///             .build(),
///     )
///     .draw(&mut display);
///
/// Rectangle::new(Point::new(24, 0), Point::new(40, 16))
///     .into_styled(
///         PrimitiveStyleBuilder::new()
///             .stroke_color(Rgb565::new(255, 127, 0))
///             .stroke_width(1)
///             .build(),
///     )
///     .draw(&mut display);
///
/// Circle::new(Point::new(64, 8), 8)
///     .into_styled(
///         PrimitiveStyleBuilder::new()
///             .stroke_color(Rgb565::GREEN)
///             .stroke_width(1)
///             .build(),
///     )
///     .draw(&mut display);
///
/// image.draw(&mut display);
///
/// Text::new("Hello Rust!", Point::new(24, 24))
///     .into_styled(
///         TextStyleBuilder::new(Font6x8)
///             .text_color(Rgb565::RED)
///             .build(),
///     )
///     .draw(&mut display);
///
/// // Render graphics objects to the screen
/// display.flush().unwrap();
/// ```
///
/// [`embedded-graphics`]: https://crates.io/crates/embedded-graphics
pub struct Ssd1331<SPI, DC> {
    /// Pixel buffer
    ///
    /// The display is 16BPP RGB565, so two `u8`s are used for each pixel value
    buffer: [u8; BUF_SIZE],

    /// Which display rotation to use
    display_rotation: DisplayRotation,

    /// SPI interface
    spi: SPI,

    /// Data/Command pin
    dc: DC,
}

impl<SPI, DC, CommE, PinE> Ssd1331<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8, Error = CommE>,
    DC: OutputPin<Error = PinE>,
{
    /// Create new display instance
    ///
    /// Ensure `display.init()` is called before sending data otherwise nothing will be shown.
    ///
    /// The driver allocates a buffer of 96px * 64px * 16bits = 12,288 bytes. This may be too large
    /// for some target hardware.
    ///
    /// # Examples
    ///
    /// ## Create a display instance with no rotation
    ///
    /// ```rust
    /// # use ssd1331::test_helpers::{Pin, Spi};
    /// use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
    ///
    /// // Set up SPI interface and digital pin. These are stub implementations used in examples.
    /// let spi = Spi;
    /// let dc = Pin;
    ///
    /// let mut display = Ssd1331::new(spi, dc, Rotate0);
    ///
    /// // Initialise and clear the display
    /// display.init().unwrap();
    /// display.flush().unwrap();
    /// ```
    pub fn new(spi: SPI, dc: DC, display_rotation: DisplayRotation) -> Self {
        Self {
            spi,
            dc,
            display_rotation,
            buffer: [0; BUF_SIZE],
        }
    }

    /// Release SPI and DC resources for reuse in other code
    pub fn release(self) -> (SPI, DC) {
        (self.spi, self.dc)
    }

    /// Clear the display buffer
    ///
    /// `display.flush()` must be called to update the display
    pub fn clear(&mut self) {
        self.buffer = [0; BUF_SIZE];
    }

    /// Reset the display
    ///
    /// This method brings the RST pin low for 1ms to reset the module,  waits for another 1ms then
    /// brings RST high
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
        delay.delay_ms(1);
        rst.set_high().map_err(Error::Pin)?;

        Ok(())
    }

    /// Send the full framebuffer to the display
    ///
    /// This resets the draw area the full size of the display
    pub fn flush(&mut self) -> Result<(), Error<CommE, PinE>> {
        // Ensure the display buffer is at the origin of the display before we send the full frame
        // to prevent accidental offsets
        self.set_draw_area((0, 0), (DISPLAY_WIDTH - 1, DISPLAY_HEIGHT - 1))?;

        // 1 = data, 0 = command
        self.dc.set_high().map_err(Error::Pin)?;

        self.spi.write(&self.buffer).map_err(Error::Comm)?;

        Ok(())
    }

    /// Set the top left and bottom right corners of a bounding box to draw to
    pub fn set_draw_area(
        &mut self,
        start: (u8, u8),
        end: (u8, u8),
    ) -> Result<(), Error<CommE, PinE>> {
        Command::ColumnAddress(start.0, end.0).send(&mut self.spi, &mut self.dc)?;
        Command::RowAddress(start.1.into(), (end.1).into()).send(&mut self.spi, &mut self.dc)?;
        Ok(())
    }

    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let idx = match self.display_rotation {
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

    /// Initialise display, setting sensible defaults and rotation
    pub fn init(&mut self) -> Result<(), Error<CommE, PinE>> {
        let display_rotation = self.display_rotation;

        Command::DisplayOn(false).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayClockDiv(0xF, 0x0).send(&mut self.spi, &mut self.dc)?;
        Command::Multiplex(DISPLAY_HEIGHT - 1).send(&mut self.spi, &mut self.dc)?;
        Command::StartLine(0).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayOffset(0).send(&mut self.spi, &mut self.dc)?;

        self.set_rotation(display_rotation)?;

        // Values taken from [here](https://github.com/adafruit/Adafruit-SSD1331-OLED-Driver-Library-for-Arduino/blob/master/Adafruit_SSD1331.cpp#L119-L124)
        Command::Contrast(0x91, 0x50, 0x7D).send(&mut self.spi, &mut self.dc)?;
        Command::PreChargePeriod(0x1, 0xF).send(&mut self.spi, &mut self.dc)?;
        Command::VcomhDeselect(VcomhLevel::V071).send(&mut self.spi, &mut self.dc)?;
        Command::AllOn(false).send(&mut self.spi, &mut self.dc)?;
        Command::Invert(false).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayOn(true).send(&mut self.spi, &mut self.dc)?;

        Ok(())
    }

    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// # Examples
    ///
    /// ## No rotation
    ///
    /// ```rust
    /// # use ssd1331::test_helpers::{Spi, Pin};
    /// use ssd1331::{DisplayRotation, Ssd1331};
    ///
    /// // Set up SPI interface and digital pin. These are stub implementations used in examples.
    /// let spi = Spi;
    /// let dc = Pin;
    ///
    /// let display = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);
    ///
    /// assert_eq!(display.dimensions(), (96, 64));
    /// ```
    ///
    /// ## 90 degree rotation rotation
    ///
    /// ```rust
    /// # use ssd1331::test_helpers::{Spi, Pin};
    /// use ssd1331::{DisplayRotation, Ssd1331};
    ///
    /// // Set up SPI interface and digital pin. These are stub implementations used in examples.
    /// let spi = Spi;
    /// let dc = Pin;
    ///
    /// let display = Ssd1331::new(spi, dc, DisplayRotation::Rotate90);
    ///
    /// assert_eq!(display.dimensions(), (64, 96));
    /// ```
    pub fn dimensions(&self) -> (u8, u8) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                (DISPLAY_WIDTH, DISPLAY_HEIGHT)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                (DISPLAY_HEIGHT, DISPLAY_WIDTH)
            }
        }
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), Error<CommE, PinE>> {
        self.display_rotation = rot;

        match rot {
            DisplayRotation::Rotate0 => {
                Command::RemapAndColorDepth(
                    false,
                    false,
                    ColorMode::CM65k,
                    AddressIncrementMode::Horizontal,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate90 => {
                Command::RemapAndColorDepth(
                    true,
                    false,
                    ColorMode::CM65k,
                    AddressIncrementMode::Vertical,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate180 => {
                Command::RemapAndColorDepth(
                    true,
                    true,
                    ColorMode::CM65k,
                    AddressIncrementMode::Horizontal,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate270 => {
                Command::RemapAndColorDepth(
                    false,
                    true,
                    ColorMode::CM65k,
                    AddressIncrementMode::Vertical,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
        };

        Ok(())
    }

    /// Get the current rotation of the display
    pub fn rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Turn the display on (eg exiting sleep mode)
    pub fn turn_on(&mut self) -> Result<(), Error<CommE, PinE>> {
        Command::DisplayOn(true).send(&mut self.spi, &mut self.dc)
    }

    /// Turn the display off (enter sleep mode)
    pub fn turn_off(&mut self) -> Result<(), Error<CommE, PinE>> {
        Command::DisplayOn(false).send(&mut self.spi, &mut self.dc)
    }
}

#[cfg(feature = "graphics")]
use core::convert::TryInto;
#[cfg(feature = "graphics")]
use embedded_graphics::{
    drawable,
    geometry::Size,
    pixelcolor::{
        raw::{RawData, RawU16},
        Rgb565,
    },
    DrawTarget,
};

#[cfg(feature = "graphics")]
impl<SPI, DC> DrawTarget<Rgb565> for Ssd1331<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
{
    type Error = core::convert::Infallible;

    fn draw_pixel(&mut self, pixel: drawable::Pixel<Rgb565>) -> Result<(), Self::Error> {
        let drawable::Pixel(pos, color) = pixel;

        // Guard against negative values. All positive i32 values from `pos` can be represented in
        // the `u32`s that `set_pixel()` accepts.
        if pos.x < 0 || pos.y < 0 {
            return Ok(());
        }

        self.set_pixel(
            (pos.x).try_into().unwrap(),
            (pos.y).try_into().unwrap(),
            RawU16::from(color).into_inner(),
        );

        Ok(())
    }

    fn size(&self) -> Size {
        let (w, h) = self.dimensions();

        Size::new(w as u32, h as u32)
    }
}
