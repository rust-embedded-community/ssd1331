//! Print "Hello world!" with "Hello rust!" underneath. Uses the `embedded_graphics` crate to draw
//! the text with a 6x8 pixel font.
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
//! Run on a Blue Pill with `cargo run --example text`.

#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    egtext,
    fonts::{Font6x8, Text},
    geometry::Point,
    pixelcolor::Rgb565,
    prelude::*,
    style::TextStyleBuilder,
    text_style,
};
use panic_semihosting as _;
use ssd1331::{DisplayRotation::Rotate0, Ssd1331};
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

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

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
        &mut rcc.apb2,
    );

    let mut disp = Ssd1331::new(spi, dc, Rotate0);

    disp.reset(&mut rst, &mut delay).unwrap();
    disp.init().unwrap();
    disp.flush().unwrap();

    // Red with a small amount of green creates a deep orange colour
    let rust = Rgb565::new(0xff, 0x07, 0x00);

    Text::new("Hello world!", Point::zero())
        .into_styled(
            TextStyleBuilder::new(Font6x8)
                .text_color(Rgb565::WHITE)
                .build(),
        )
        .draw(&mut disp);

    Text::new("Hello Rust!", Point::new(0, 16))
        .into_styled(TextStyleBuilder::new(Font6x8).text_color(rust).build())
        .draw(&mut disp);

    // Macros can also be used
    egtext!(
        text = "Hello macros!",
        top_left = (0, 0),
        style = text_style!(
            font = Font6x8,
            text_color = Rgb565::RED,
            background_color = Rgb565::GREEN
        )
    )
    .translate(Point::new(0, 24))
    .draw(&mut disp);

    disp.flush().unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
