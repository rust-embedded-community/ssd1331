//! Draw the Rust logo centered on a 90 degree rotated 128x64px display
//!
//! Image was created with ImageMagick:
//!
//! ```bash
//! convert rust.png -depth 1 gray:rust.raw
//! ```
//!
//! This example is for the STM32F103 "Blue Pill" board using a 4 wire interface to the display on
//! SPI1.
//!
//! Wiring connections are as follows
//!
//! ```
//! GND -> GND
//! 3V3 -> VCC
//! PA5 -> SCL
//! PA7 -> SDA
//! PB0 -> RST
//! PB1 -> D/C
//! ```
//!
//! Run on a Blue Pill with `cargo run --release --example rotation`.

#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{image::ImageRawLE, pixelcolor::BinaryColor, prelude::*};
use panic_semihosting as _;
use ssd1331::{DisplayRotation, Ssd1331};
use stm32f1xx_hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    stm32,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();

    // SPI1
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let dc = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        8.mhz(),
        clocks,
    );

    // Initialise the display with a default rotation of 90 degrees
    let mut display = Ssd1331::new(spi, dc, DisplayRotation::Rotate90);

    display.reset(&mut rst, &mut delay).unwrap();
    display.init().unwrap();
    display.flush().unwrap();

    // Set a new rotation of 270 degrees
    display.set_rotation(DisplayRotation::Rotate270).unwrap();

    // Load a 1BPP 64x64px image with LE (Little Endian) encoding of the Rust logo, white foreground
    // black background
    let im = ImageRawLE::<BinaryColor>::new(include_bytes!("./rust.raw"), 64);

    // Use `color_converted` to create a wrapper that converts BinaryColors to Rgb565 colors to send
    // to the display.
    im.draw(&mut display.color_converted()).unwrap();

    display.flush().unwrap();

    loop {}
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
