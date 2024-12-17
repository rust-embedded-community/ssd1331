//! Helpers for use in examples and tests

use hal::{digital::OutputPin, spi::SpiDevice};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Spi;

impl SpiDevice for Spi {
    fn transaction(
        &mut self,
        _operations: &mut [hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.transaction(&mut [hal::spi::Operation::Transfer(read, write)])
    }
}

impl hal::spi::ErrorType for Spi {
    type Error = hal::spi::ErrorKind;
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Pin;

impl hal::digital::ErrorType for Pin {
    type Error = hal::digital::ErrorKind;
}

impl OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
