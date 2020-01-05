# SSD1331 driver

[![Build Status](https://travis-ci.org/jamwaffles/ssd1331.svg?branch=master)](https://travis-ci.org/jamwaffles/ssd1331)

[![SSD1331 display showing Ferris](readme_banner.jpg?raw=true)](examples/image.rs)

SPI (4 wire) driver for the SSD1331 OLED display.

<!-- See the [announcement blog post](https://wapl.es/electronics/rust/2018/04/30/ssd1331-driver.html) for more information. -->

The display is configured by this driver to use a 16 bit, R5 G6 B5 pixel definition.
You can convert images into the correct BMP format with the following commands:

```bash
convert my_image.png -flip -type truecolor -define bmp:subtype=RGB565 -depth 16 -strip my_image.bmp
```

You can also export images directly from The GIMP by saving as `.bmp` and choosing the following option:

![The GIMP RGB565 export option.](readme_gimp_export.png?raw=true)

## [Documentation](https://docs.rs/ssd1331)

## [Examples](examples)

Load a BMP image of the Rust logo and display it in the center of the display. From
[`examples/bmp.rs`](examples/bmp.rs):

```rust,no_run
#![no_std]
#![no_main]

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::{geometry::Point, image::ImageBmp, prelude::*};
use panic_semihosting as _;
use ssd1331::Builder;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f1xx_hal::stm32;

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

    let mut disp = Builder::new().connect_spi(spi, dc);

    disp.reset(&mut rst, &mut delay).unwrap();
    disp.init().unwrap();
    disp.flush().unwrap();

    let im = ImageBmp::new(include_bytes!("../examples/rust-pride.bmp")).unwrap();

    let moved = im.translate(Point::new((96 - im.width() as i32) / 2, 0));

    disp.draw(moved.into_iter());

    disp.flush().unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
```

![Rust rainbow demo image.](readme_pride.jpg?raw=true)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
