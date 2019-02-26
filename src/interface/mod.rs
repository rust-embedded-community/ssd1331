//! SSD1331 Communication Interface (SPI)
//!
//! This is used by the [builder](../builder/index.html) method
//! [connect_spi](../builder/struct.Builder.html#method.connect_spi).
//!
//! The types that these interfaces define are quite lengthy, so it is recommended that you create
//! a type alias. Here's an example for SPI1 on an STM32F103xx:
//!
//! ```rust
//! # extern crate ssd1331;
//! # extern crate stm32f103xx_hal as hal;
//! # use hal::gpio::gpioa::{PA5, PA6, PA7};
//! # use hal::gpio::gpiob::PB1;
//! # use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
//! # use hal::spi::Spi;
//! # use hal::stm32f103xx::SPI1;
//! # use ssd1331::interface::SpiInterface;
//! pub type OledDisplay = GraphicsMode<
//!     SpiInterface<
//!         Spi<
//!             SPI1,
//!             (
//!                 PA5<Alternate<PushPull>>,
//!                 PA6<Input<Floating>>,
//!                 PA7<Alternate<PushPull>>,
//!             ),
//!         >,
//!         PB1<Output<PushPull>>,
//!     >,
//! >;
//! ```
//!
//! [Example](https://github.com/jamwaffles/ssd1331/blob/master/examples/blinky.rs)

pub mod spi;

/// A method of communicating with SSD1331
pub trait DisplayInterface {
    /// Send a batch of up to 8 commands to display.
    fn send_commands(&mut self, cmd: &[u8]) -> Result<(), ()>;
    /// Send data to display.
    fn send_data(&mut self, buf: &[u8]) -> Result<(), ()>;
}

pub use self::spi::SpiInterface;
