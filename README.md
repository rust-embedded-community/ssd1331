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

Load a BMP and display it in the center of the display. From [`examples/bmp.rs`](examples/bmp.rs):

```rust
#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::image::Image1BPP;
use embedded_graphics::prelude::*;
use hal::i2c::{BlockingI2c, DutyCycle, Mode};
use hal::prelude::*;
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

    disp.init().unwrap();
    disp.flush().unwrap();

    // 16BPP, RGB565-format BMP image
    let im = ImageBmp::new(include_bytes!("./rust-pride.bmp")).unwrap();

    let centered = im.translate(Coord::new((96 - im.width() as i32) / 2, 0));

    disp.draw(centered.into_iter());

    disp.draw(im.into_iter());
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
