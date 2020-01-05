use crate::display::Ssd1331;
use crate::displayrotation::DisplayRotation;
use crate::properties::Properties;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

/// SSD1331 factory
///
/// This is the easiest way to create a driver instance. Configuration parameters such as display
/// rotation can be set. The builder will be consumed and will return an instance of the [`Ssd1331`]
/// struct bound to the given SPI interface when `.connect_spi()` is called.
///
/// # Examples
///
/// ## Connect to display with default rotation (0 deg)
///
/// ```rust
/// # use ssd1331::test_helpers::{Spi, Pin};
/// use ssd1331::Builder;
///
/// // Set up SPI interface and digital pin. These are stub implementations used in examples.
/// let spi = Spi;
/// let dc = Pin;
///
/// let display = Builder::new().connect_spi(spi, dc);
/// ```
///
/// ## Connect to display with rotation of 90 deg
///
/// ```rust
/// # use ssd1331::test_helpers::{Spi, Pin};
/// use ssd1331::{DisplayRotation,Builder};
///
/// // Set up SPI interface and digital pin. These are stub implementations used in examples.
/// let spi = Spi;
/// let dc = Pin;
///
/// let display = Builder::new().rotation(DisplayRotation::Rotate90).connect_spi(spi, dc);
/// ```
///
/// [`Ssd1331`]: ../struct.Ssd1331.html
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
