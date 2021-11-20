//! The rust-toolchain will pull in the correct nightly and target so all you
//! need to run is 
//!
//! cargo run --release
//!
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::future::pending;
use embassy::interrupt::InterruptExt;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::gpio::{self, AnyPin, Level, NoPin, Output, OutputDrive, Pin};
use embassy_nrf::{interrupt, spim};
use embedded_graphics::prelude::*;
use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use embedded_hal::digital::v2::OutputPin;
use ssd1331::{DisplayRotation, Ssd1331};
use tinybmp::Bmp;

// we make a lazily created static
static EXECUTOR: Forever<embassy::executor::Executor> = Forever::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    // once we hit runtime we create and fill that executor finally
    let executor = EXECUTOR.put(embassy::executor::Executor::new());

    // provides the peripherals from the async first pac if you selected it
    let dp = embassy_nrf::init(Default::default());

    let green = gpio::Output::new(
        // degrade just a typesystem hack to forget which pin it is so we can
        // call it Anypin and make our function calls more generic
        dp.P0_22.degrade(),
        gpio::Level::High,
        gpio::OutputDrive::Standard,
    );

    // spawn tasks
    executor.run(|spawner| {
        let _ = spawner.spawn(blinky_task(green));
        let _ = spawner.spawn(display_task());
    })
}

#[embassy::task]
async fn blinky_task(mut green: gpio::Output<'static, AnyPin>) {
    loop {
        green.set_high().unwrap();
        Timer::after(Duration::from_millis(300)).await;
        green.set_low().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy::task]
pub async fn display_task() {
    // Too lazy to pass all the pins and peripherals we need.
    // Safety: Fragile but safe as long as pins and peripherals arent used
    // anywhere else
    let mut dp = unsafe { <embassy_nrf::Peripherals as embassy::util::Steal>::steal() };

    let mut spim_irq = interrupt::take!(SPIM3);
    spim_irq.set_priority(interrupt::Priority::P4);

    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M16;
    let spim = spim::Spim::new(
        &mut dp.SPI3,
        &mut spim_irq,
        &mut dp.P0_21,
        NoPin,
        &mut dp.P0_17,
        spim_config,
    );

    let mut rst = Output::new(&mut dp.P0_16, Level::High, OutputDrive::Standard);
    let dc = Output::new(&mut dp.P0_15, Level::High, OutputDrive::Standard);
    let mut display = Ssd1331::new(spim, dc, DisplayRotation::Rotate0);
    Timer::after(Duration::from_millis(1)).await;
    rst.set_low().ok();
    Timer::after(Duration::from_millis(1)).await;
    rst.set_high().ok();
    display.init().unwrap();

    let (w, h) = display.dimensions();

    let bmp =
        Bmp::from_slice(include_bytes!("../../../assets/rust-pride.bmp")).expect("Failed to load BMP image");

    let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::zero());

    // Position image in the center of the display
    let moved = im.translate(Point::new(
        (w as u32 - bmp.size().width) as i32 / 2,
        (h as u32 - bmp.size().height) as i32 / 2,
    ));

    moved.draw(&mut display).unwrap();

    display.flush_async().await.unwrap();
    // display.flush().unwrap();

    // Block forever so the above drivers don't get dropped
    pending::<()>().await;
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
