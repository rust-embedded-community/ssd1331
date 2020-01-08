//! SSD1331 OLED display driver
//!
//! This crate is an SPI-based driver for the popular SSD1331 colour OLED display. This display uses
//! an RGB565 colour space on a canvas of 96x64 pixels and runs over SPI. This driver should work
//! with any device implementing the [embedded-hal] [`blocking::spi::Write`] trait.
//!
//! [`embedded-graphics`] is also supported behind the `graphics` feature flag (on by default).
//!
//! Note that the driver requires at least 12288 bytes (96 x 64 pixels, 16 bits per pixel) of memory
//! to store the display's framebuffer.
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
//! use embedded_graphics::{
//!     pixelcolor::{
//!         raw::{RawData, RawU16},
//!         Rgb565,
//!     },
//!     prelude::*,
//! };
//! use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
//!
//! // Set up SPI interface and digital pin. These are stub implementations used in examples.
//! let spi = Spi;
//! let dc = Pin;
//!
//! let mut display = Ssd1331::new(spi, dc, Rotate0);
//! display.init();
//!
//! // Use raw hex values
//! display.set_pixel(10, 20, 0xf00);
//! // Or embedded-graphics' `Rgb565` if the `graphics` feature is enabled
//! display.set_pixel(10, 30, RawU16::from(Rgb565::new(255, 127, 0)).into_inner());
//!
//! display.flush();
//! ```
//!
//! ## Render a rainbow Rust logo
//!
//! ```rust
//! # use ssd1331::test_helpers::{Spi, Pin};
//! use embedded_graphics::{image::ImageBmp, prelude::*};
//! use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
//!
//! // Set up SPI interface and digital pin. These are stub implementations used in examples.
//! let spi = Spi;
//! let dc = Pin;
//!
//! let mut display = Ssd1331::new(spi, dc, Rotate0);
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
//!
//! # Features
//!
//! ## `graphics` (enabled by default)
//!
//! Enable the `graphics` feature in `Cargo.toml` to get access to features in the
//! [`embedded-graphics`] crate. This adds the `.draw()` method to the [`Ssd1331`] struct which
//! accepts any `embedded-graphics` compatible item.
//!
//! [embedded-hal]: https://docs.rs/embedded-hal
//! [`blocking::spi::Write`]: https://docs.rs/embedded-hal/0.2.3/embedded_hal/blocking/spi/trait.Write.html
//! [`Ssd1331`]: ./struct.Ssd1331.html
//! [`embedded-graphics`]: https://docs.rs/embedded-graphics

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

mod check_readme;
mod command;
mod display;
mod displayrotation;
mod error;
#[doc(hidden)]
pub mod test_helpers;

pub use crate::{display::Ssd1331, displayrotation::DisplayRotation, error::Error};
