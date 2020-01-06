//! Helpers for use in examples and tests

use embedded_hal::{
    blocking::spi::{self, Transfer},
    digital::v2::OutputPin,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Spi;

impl spi::Write<u8> for Spi {
    type Error = ();

    fn write(&mut self, _buf: &[u8]) -> Result<(), ()> {
        Ok(())
    }
}

impl Transfer<u8> for Spi {
    type Error = ();

    fn transfer<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], ()> {
        Ok(buf)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Pin;

impl OutputPin for Pin {
    type Error = ();

    fn set_high(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), ()> {
        Ok(())
    }
}
