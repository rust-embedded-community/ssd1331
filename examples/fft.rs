//! A simple FFT meter using the [Adafruit MAX9184 module](https://www.adafruit.com/product/1713)
//!
//! This example is for the STM32F103 "Blue Pill" board using a 4 wire interface to the display on
//! SPI1.
//!
//! Run on a Blue Pill with `cargo run --example fft --release`.

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    fonts::{Font6x8, Text},
    geometry::Point,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle},
};
use panic_semihosting as _;
use ssd1331::{DisplayRotation, Ssd1331};
use stm32f1xx_hal::{
    adc,
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

    // Configure system ADC clocks. Blue Pill has an 8MHz external crystal which we'll PLL up to
    // 72MHz for teh speeds
    let clocks = rcc
        .cfgr
        .adcclk(2.mhz())
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let mut delay = Delay::new(cp.SYST, clocks);

    // Display reset line
    let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

    // Display
    let mut disp = {
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

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

        Ssd1331::new(spi, dc, DisplayRotation::Rotate180)
    };

    // ADC
    let mut adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);
    let mut mic = gpioa.pa3.into_analog(&mut gpioa.crl);

    disp.reset(&mut rst, &mut delay).unwrap();
    disp.init().unwrap();
    disp.flush().unwrap();

    // Red with a small amount of green creates a deep orange colour
    let rust = Rgb565::new(0xff, 0x07, 0x00);

    let mut buf = heapless::String::<heapless::consts::U16>::new();

    let mut sample: u16 = 0;

    let (w, h) = disp.dimensions();

    // Re-cast u8 to u16 for convenience
    let w = w as u16;
    let h = h as u16;

    loop {
        disp.clear();

        let data: u16 = adc1.read(&mut mic).unwrap();

        buf.clear();
        write!(buf, "ADC: {}", data).ok();

        Text::new(&buf, Point::zero())
            .into_styled(TextStyle::new(Font6x8, Rgb565::WHITE))
            .draw(&mut disp);

        buf.clear();
        write!(buf, "Sample: {}", sample).ok();

        Text::new(&buf, Point::new(0, 8))
            .into_styled(TextStyle::new(Font6x8, rust))
            .draw(&mut disp);

        let bar_width = data / (4096 / w);

        Rectangle::new(
            Point::new(0, h as i32 - 5),
            Point::new(bar_width as i32, h as i32),
        )
        .into_styled(PrimitiveStyle::with_fill(rust))
        .draw(&mut disp);

        disp.flush().unwrap();

        sample += 1;
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
