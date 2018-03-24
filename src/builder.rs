//! Interface factory

use hal;
use hal::digital::OutputPin;

use super::interface::SpiInterface;
use super::SSD1331;

/// Communication interface factory
#[derive(Clone, Copy)]
pub struct Builder {}

impl Builder {
    /// Create new builder for default size of 128 x 64 pixels.
    pub fn new() -> Self {
        Self {}
    }

    /// Create spi communication interface
    pub fn connect_spi<SPI, DC>(&self, spi: SPI, dc: DC) -> SSD1331<SpiInterface<SPI, DC>>
    where
        SPI: hal::blocking::spi::Transfer<u8> + hal::blocking::spi::Write<u8>,
        DC: OutputPin,
    {
        SSD1331::new(SpiInterface::new(spi, dc))
    }
}
