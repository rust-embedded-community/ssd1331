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

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use hal::delay::Delay;
use hal::prelude::*;
use hal::spi::{Mode, Phase, Polarity, Spi};
use hal::stm32;
use ssd1331::prelude::*;
use ssd1331::Builder;

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

    let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

    disp.reset(&mut rst, &mut delay);
    disp.init().unwrap();
    disp.flush().unwrap();

    disp.draw(
        Font6x8::render_str("Hello world!")
            .with_stroke(Some(1u8.into()))
            .into_iter(),
    );
    disp.draw(
        Font6x8::render_str("Hello Rust!")
            .with_stroke(Some(1u8.into()))
            .translate(Coord::new(0, 16))
            .into_iter(),
    );

    disp.flush().unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
