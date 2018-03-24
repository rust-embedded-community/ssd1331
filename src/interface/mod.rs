//! SSD1331 Communication Interface

pub mod spi;

/// A method of communicating with SSD1331
pub trait DisplayInterface {
    /// Communication error.
    type Error;

    /// Send a command to display.
    fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error>;
    /// Send data to display.
    fn send_data(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}

pub use self::spi::SpiInterface;
