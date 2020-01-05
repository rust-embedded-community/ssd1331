//! Interface factory
//!
//! This is the easiest way to create a driver instance. You can set various parameters of the
//! driver and give it an interface to use. The builder will return a
//! [`mode::RawMode`](../mode/raw/struct.RawMode.html) object which you should coerce to a richer
//! display mode, like [mode::Graphics](../mode/graphics/struct.GraphicsMode.html) for drawing
//! primitives and text.
//!
//! # Examples
//!
//! Connect over SPI with default rotation (0 deg) and size (128x64):
//!
//! ```rust,ignore
//! let spi = /* SPI interface from your HAL of choice */;
//! let dc = /* GPIO data/command select pin */;
//!
//! Builder::new().connect_spi(spi, dc);
//! ```

use crate::display::Ssd1331;
use crate::displayrotation::DisplayRotation;
use crate::properties::Properties;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

/// Builder struct. Driver options and interface are set using its methods.
#[derive(Clone, Copy)]
pub struct Builder {
    rotation: DisplayRotation,
}

impl Builder {
    /// Create new builder with a default size of 128 x 64 pixels and no rotation.
    pub fn new() -> Self {
        Self {
            rotation: DisplayRotation::Rotate0,
        }
    }

    /// Set the rotation of the display to one of four values. Defaults to no rotation.
    pub fn rotation(&self, rotation: DisplayRotation) -> Self {
        Self { rotation, ..*self }
    }

    /// Finish the builder and use SPI to communicate with the display
    pub fn connect_spi<SPI, DC>(&self, spi: SPI, dc: DC) -> Ssd1331<SPI, DC>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        DC: OutputPin,
    {
        Ssd1331::new(Properties::new(spi, dc, self.rotation))
    }
}
