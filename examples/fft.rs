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
use cortex_m_semihosting::hprintln;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    geometry::Point,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle},
};
use num_traits::cast::ToPrimitive;
use num_traits::float::FloatCore;
use panic_semihosting as _;
use ssd1331::{DisplayRotation, Ssd1331};
use stm32f1xx_hal::{
    adc,
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    stm32,
};

fn add_sample(buf: &mut [f32], sample: u16) {
    // Half the mic input range in ADC value
    let sample_range_half = 4096.0 * (MAX - MIN) / 2.0;
    let adc_bias = BIAS * 4096.0;

    buf.rotate_left(1);

    // Subtract bias so sample is centered around 0
    let sample = sample as f32 - adc_bias;

    // Scale sample from -1 to 1
    let sample = sample / sample_range_half;

    buf[buf.len() - 1] = sample;
}

// Mic output has a DC bias of 1.25v. Assuming 3.3v supply voltage.
const BIAS: f32 = (1.25 / 3.3);

// 2Vpp (peak to peak) so 1v above bias voltage, or 2.25v
const MAX: f32 = (2.25 / 3.3);
const MIN: f32 = (0.25 / 3.3);

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

    let mut samples: [f32; 32] = [0.0; 32];

    // Bias point of 1.25 volts, mapped to screen coordinates
    let bias_x = (BIAS * w as f32) as u32;

    loop {
        disp.clear();

        let data: u16 = adc1.read(&mut mic).unwrap();

        add_sample(&mut samples, data);

        // let mut out = samples.clone();
        // let spectrum = microfft::real::rfft_32(&mut out);

        // spectrum
        //     .iter()
        //     .map(|value| value.scale(0.01).re.abs() as i32)
        //     .enumerate()
        //     .map(|(idx, value)| {
        //         Line::new(
        //             Point::new(idx as i32, h as i32 - 10),
        //             Point::new(idx as i32, h as i32 - 12 - value),
        //         )
        //         .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1))
        //         .into_iter()
        //     })
        //     .flatten()
        //     .draw(&mut disp);

        buf.clear();
        // write!(buf, "Sample: {}", samples.last().unwrap()).ok();
        write!(buf, "Sample: {}", data).ok();

        Text::new(&buf, Point::zero())
            .into_styled(TextStyle::new(Font6x8, Rgb565::WHITE))
            .draw(&mut disp);

        let bar_width = data / (4096 / w);

        Rectangle::new(
            Point::new(w as i32 / 2, h as i32 - 8),
            Point::new(
                (samples.last().unwrap() * (w / 2) as f32) as i32,
                h as i32 - 2,
            ),
        )
        .into_styled(PrimitiveStyle::with_fill(rust))
        .draw(&mut disp);

        // // Bias point line
        // Line::new(
        //     Point::new(bias_x as i32, h as i32),
        //     Point::new(bias_x as i32, h as i32 - 10),
        // )
        // .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1))
        // .draw(&mut disp);

        disp.flush().unwrap();

        sample += 1;
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
