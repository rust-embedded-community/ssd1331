//! TODO: Docs

#![no_std]
#![no_main]

use core::convert::TryFrom;
use cortex_m::singleton;
use cortex_m_semihosting::hprintln;
use embedded_graphics::{
    geometry::Point, image::Image, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use panic_semihosting as _;
use rtfm::app;
use ssd1331::{DisplayRotation::Rotate180, Ssd1331};
use stm32f1xx_hal::{
    adc,
    delay::Delay,
    dma, gpio,
    pac::{self, ADC1, SPI1},
    prelude::*,
    spi::{self, Mode, Phase, Polarity, Spi},
    timer,
    timer::{CountDownTimer, Event, Timer},
};
use tinybmp::Bmp;

type Display = Ssd1331<
    spi::Spi<
        SPI1,
        spi::Spi1NoRemap,
        (
            gpio::gpioa::PA5<gpio::Alternate<gpio::PushPull>>,
            gpio::gpioa::PA6<gpio::Input<gpio::Floating>>,
            gpio::gpioa::PA7<gpio::Alternate<gpio::PushPull>>,
        ),
    >,
    gpio::gpiob::PB1<gpio::Output<gpio::PushPull>>,
>;

type DebugLed = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

type Mic = stm32f1xx_hal::gpio::gpioa::PA3<stm32f1xx_hal::gpio::Analog>;

// type AdcChannelThing =
//     dma::RxDma<adc::AdcPayload<gpio::gpioa::PA3<gpio::Analog>, adc::Continuous>, dma::dma1::C1>;

// type Sampler = stm32f1xx_hal::dma::CircBuffer<[u16; NUM_SAMPLES], AdcChannelThing>;
type Sampler = stm32f1xx_hal::adc::Adc<ADC1>;

const NUM_SAMPLES: usize = 8;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        display: Display,
        timer: CountDownTimer<pac::TIM1>,
        led: DebugLed,
        #[init(0)]
        count: u32,
        #[init([0; NUM_SAMPLES])]
        sample_buf: [u16; NUM_SAMPLES],
        sampler: Sampler,
        mic: Mic,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let dp = cx.device;
        let core = cx.core;

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

        // SPI1
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        let mut delay = Delay::new(core.SYST, clocks);

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

        let mut display = Ssd1331::new(spi, dc, Rotate180);

        display.reset(&mut rst, &mut delay).unwrap();
        display.init().unwrap();

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        let mut timer = Timer::tim1(dp.TIM1, &clocks, &mut rcc.apb2).start_count_down(20.hz());

        timer.listen(timer::Event::Update);

        let mut sampler = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);

        let mic = gpioa.pa3.into_analog(&mut gpioa.crl);

        // Init the static resources to use them later through RTFM
        init::LateResources {
            timer,
            display,
            led,
            sampler,
            mic,
        }
    }

    // #[task(binds = DMA1_CHANNEL1, priority = 2, resources = [count, sample_buf, adc])]
    // fn sample(cx: sample::Context) {
    //     let sample::Resources {
    //         count,
    //         sample_buf,
    //         adc,
    //         ..
    //     } = cx.resources;

    //     // *sample_buf = adc.peek(|half, _| *half).unwrap();

    //     *count += 1;
    // }

    #[task(binds = TIM1_UP, resources = [count, timer, led, sample_buf, mic, sampler])]
    fn update(cx: update::Context) {
        use core::fmt::Write;

        let update::Resources {
            count,
            timer,
            led,
            sample_buf,
            sampler,
            mic,
            ..
        } = cx.resources;

        sample_buf.rotate_right(1);

        sample_buf[sample_buf.len() - 1] = sampler.read(mic).unwrap();

        led.toggle();

        // Clears the update flag
        timer.clear_update_interrupt_flag();
    }
};
