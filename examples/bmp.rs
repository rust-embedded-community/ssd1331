//! Draw a colourful Rust logo on an SSD1331 display over SPI
//!
//! Image was exported with The GIMP. Export as `*.bmp` and use the 16 bit "R5 G6 B5" option under
//! "advanced options".
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
//! Run on a Blue Pill with `cargo +nightly run --release --example bmp --features=async`.

#![no_std]
#![no_main]

use core::future::Future;
use core::ptr::null_mut;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{geometry::Point, image::Image, pixelcolor::Rgb565, prelude::*};
use panic_semihosting as _;
use ssd1331::{DisplayRotation, Ssd1331};
use stm32f1xx_hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    stm32,
};
use tinybmp::Bmp;

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

    let mut display = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);

    display.reset(&mut rst, &mut delay).unwrap();
    display.init().unwrap();
    display.flush().unwrap();

    let (w, h) = display.dimensions();

    let bmp =
        Bmp::from_slice(include_bytes!("./rust-pride.bmp")).expect("Failed to load BMP image");

    let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::zero());

    // Position image in the center of the display
    let moved = im.translate(Point::new(
        (w as u32 - bmp.size().width) as i32 / 2,
        (h as u32 - bmp.size().height) as i32 / 2,
    ));

    moved.draw(&mut display).unwrap();

    block_on(display.flush_async()).unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn waker_nop(_p: *const ()) {}

static VTABLE: RawWakerVTable = RawWakerVTable::new(waker_clone, waker_nop, waker_nop, waker_nop);

pub fn block_on<F: Future>(future: F) -> F::Output {
    futures::pin_mut!(future);
    let waker = &unsafe { Waker::from_raw(RawWaker::new(null_mut(), &VTABLE)) };
    let mut cx = Context::from_waker(waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

impl<'d, T: Instance> embassy_traits::spi::Write<u8> for Spim<'d, T> {
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output=Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
        self.read_write(&mut [], data)
    }
}
