//! Draw a square, circle and triangle on the screen using the embedded_graphics library over a 4
//! wire SPI interface.
//!
//! This example is for the STM32F411 Nucleo development board.
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
//! Run it with `cargo run --example graphics`.

#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    geometry::Point,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, Rectangle, Triangle},
};
use panic_semihosting as _;
use ssd1331::{DisplayRotation, Ssd1331};
use stm32f4xx_hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, NoMiso, Phase, Polarity, Spi},
    stm32,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    // Set up the system clock to 48MHz
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    let mut gpiob = dp.GPIOB.split();

    // SPI1
    let sck = gpiob.pb3.into_alternate_af5();
    let mosi = gpiob.pb5.into_alternate_af5();

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst = gpiob.pb10.into_push_pull_output();
    let mut dc = gpiob.pb6.into_push_pull_output();

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, NoMiso, mosi),
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1.mhz().into(),
        clocks,
    );

    let mut disp = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);

    disp.reset(&mut rst, &mut delay).unwrap();
    disp.init().unwrap();
    disp.flush().unwrap();

    let (w, h) = disp.dimensions();

    // Border
    disp.draw(
        Rectangle::new(Point::new(0, 0), Point::new(w as i32 - 1, h as i32 - 1))
            .stroke(Some(Rgb565::WHITE))
            .into_iter(),
    );

    disp.draw(
        Triangle::new(
            Point::new(8, 16 + 16),
            Point::new(8 + 16, 16 + 16),
            Point::new(8 + 8, 16),
        )
        .stroke(Some(Rgb565::RED))
        .into_iter(),
    );

    disp.draw(
        Rectangle::new(Point::new(36, 16), Point::new(36 + 16, 16 + 16))
            .stroke(Some(Rgb565::GREEN))
            .into_iter(),
    );

    disp.draw(
        Circle::new(Point::new(72, 16 + 8), 8)
            .stroke(Some(Rgb565::BLUE))
            .into_iter(),
    );

    disp.flush().unwrap();

    loop {}
}
