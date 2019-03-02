//! SSD1331 OLED display driver
//!
//! The driver must be initialised by passing an  SPI interface peripheral to the [`Builder`],
//! which will in turn create a driver instance in a particular mode. By default, the builder
//! returns a [`mode::RawMode`] instance which isn't very useful by itself. You can coerce the driver
//! into a more useful mode by calling `into()` and defining the type you want to coerce to. For
//! example, to initialise the display with an I2CSPI interface and [mode::GraphicsMode], you would do
//! something like this:
//!
//! ```rust,ignore
//! let spi = Spi::spi1(/* snip */);
//!
//! let mut disp: GraphicsMode<_> = Builder::new().connect_spi(dc, spi).into();
//! disp.init();
//!
//! disp.set_pixel(10, 20, 1);
//! ```
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
//! Examples can be found in
//! [the examples/ folder](https://github.com/jamwaffles/ssd1331/blob/master/examples)
//!
//! ## Draw some text to the display
//!
//! Uses [mode::GraphicsMode] and [embedded_graphics](../embedded_graphics/index.html).
//!
//! ```rust,no-run
//! #![no_std]
//! #![no_main]
//!
//! extern crate cortex_m;
//! extern crate cortex_m_rt as rt;
//! extern crate panic_semihosting;
//! extern crate stm32f1xx_hal as hal;
//!
//! use cortex_m_rt::ExceptionFrame;
//! use cortex_m_rt::{entry, exception};
//! use embedded_graphics::fonts::Font6x8;
//! use embedded_graphics::prelude::*;
//! use hal::i2c::{BlockingI2c, DutyCycle, Mode};
//! use hal::prelude::*;
//! use hal::stm32;
//! use ssd1331::prelude::*;
//! use ssd1331::Builder;
//!
//! #[entry]
//! fn main() -> ! {
//!     let cp = cortex_m::Peripherals::take().unwrap();
//!     let dp = stm32::Peripherals::take().unwrap();
//!
//!     let mut flash = dp.FLASH.constrain();
//!     let mut rcc = dp.RCC.constrain();
//!
//!     let clocks = rcc.cfgr.freeze(&mut flash.acr);
//!
//!     let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
//!
//!     let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
//!     let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
//!
//!     // SPI1
//!     let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
//!     let miso = gpioa.pa6;
//!     let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
//!
//!     let mut delay = Delay::new(cp.SYST, clocks);
//!
//!     let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
//!     let dc = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
//!
//!     let spi = Spi::spi1(
//!         dp.SPI1,
//!         (sck, miso, mosi),
//!         &mut afio.mapr,
//!         Mode {
//!             polarity: Polarity::IdleLow,
//!             phase: Phase::CaptureOnFirstTransition,
//!         },
//!         8.mhz(),
//!         clocks,
//!         &mut rcc.apb2,
//!     );
//!
//!     let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
//!
//!     disp.init().unwrap();
//!     disp.flush().unwrap();
//!
//!     disp.draw(
//!         Font6x8::render_str("Hello world!")
//!             .with_stroke(Some(1u8.into()))
//!             .into_iter(),
//!     );
//!     disp.draw(
//!         Font6x8::render_str("Hello Rust!")
//!             .with_stroke(Some(1u8.into()))
//!             .translate(Coord::new(0, 16))
//!             .into_iter(),
//!     );
//!
//!     disp.flush().unwrap();
//!
//!     loop {}
//! }
//! ```

#![no_std]
// TODO: Docs
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

pub mod builder;
mod command;
pub mod displayrotation;
pub mod interface;
pub mod mode;
pub mod prelude;
pub mod properties;

pub use crate::builder::Builder;
