//! SSD1331 OLED display driver
//!
//! The driver must be initialised by passing an  SPI interface peripheral to the [`Builder`],
//! which will in turn create a driver instance in a particular mode. By default, the builder
//! returns a [`mode::RawMode`] instance which isn't very useful by itself. You can coerce the driver
//! into a more useful mode by calling `into()` and defining the type you want to coerce to. For
//! example, to initialise the display with an I2CSPI interface and [mode::GraphicsMode], you would do
//! something like this:
//!
//! See the [example](https://github.com/jamwaffles/ssd1331/blob/master/examples/graphics.rs)
//! for more usage. The [entire `embedded_graphics` featureset](https://github.com/jamwaffles/embedded-graphics#features)
//! is supported by this driver.
//!
//! It's possible to customise the driver to suit your display/application. Take a look at the
//! [Builder] for available options.
//!
//! # Examples
//!
//! Full examples can be found in
//! [the examples/ folder](https://github.com/jamwaffles/ssd1331/blob/master/examples)
//!
//! ## Set individual pixels with `.set_pixel()`
//!
//! ```rust
//! # use ssd1331::test_helpers::{Spi, Pin};
//! use ssd1331::Builder;
//! use embedded_graphics::{prelude::*, pixelcolor::{raw::{RawU16, RawData}, Rgb565}};
//!
//! // Set up SPI interface and digital pin. These are stub implementations used in examples.
//! let spi = Spi;
//! let dc = Pin;
//!
//! let mut display = Builder::new().connect_spi(spi, dc);
//! display.init();
//!
//! // Use raw hex values
//! display.set_pixel(10, 20, 0xf00);
//! // Or embedded-graphics' `Rgb565`
//! display.set_pixel(10, 30, RawU16::from(Rgb565::new(255, 127, 0)).into_inner());
//!
//! display.flush();
//! ```
//!
//! ## Render a rainbow Rust logo
//!
//! ```rust
//! # use ssd1331::test_helpers::{Spi, Pin};
//! use ssd1331::Builder;
//! use embedded_graphics::{prelude::*, image::ImageBmp};
//!
//! // Set up SPI interface and digital pin. These are stub implementations used in examples.
//! let spi = Spi;
//! let dc = Pin;
//!
//! let mut display = Builder::new().connect_spi(spi, dc);
//! display.init();
//!
//! let im = ImageBmp::new(include_bytes!("../examples/rust-pride.bmp")).unwrap();
//!
//! // Center the image on the display
//! let moved = im.translate(Point::new((96 - im.width() as i32) / 2, 0));
//!
//! display.draw(moved.into_iter());
//!
//! display.flush().unwrap();
//! ```

#![no_std]
// #![deny(missing_debug_implementations)]
#![deny(missing_docs)]
// #![deny(warnings)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]

extern crate embedded_hal as hal;

const DISPLAY_WIDTH: u8 = 96;
const DISPLAY_HEIGHT: u8 = 64;

mod builder;
mod command;
mod display;
mod displayrotation;
mod properties;
#[doc(hidden)]
pub mod test_helpers;

pub use crate::builder::Builder;
pub use crate::display::Ssd1331;
pub use crate::displayrotation::DisplayRotation;
