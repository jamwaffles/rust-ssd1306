#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt; // for abort-on-panic
extern crate embedded_hal;
extern crate ssd1306;
extern crate stm32f103xx_hal as blue_pill;

use blue_pill::prelude::*;
use blue_pill::spi::Spi;
use hal::spi::{Mode, Phase, Polarity};
use ssd1306::{Resolution, Ssd1306SPI, ADDRESS};

fn main() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = blue_pill::stm32f103xx::Peripherals::take().unwrap();

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

    let rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
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

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut ssd1306 = Ssd1306SPI::new(spi1, dc, Resolution::R128x32, true);

    ssd1306.reset(&mut rst, &mut delay);
    ssd1306.init().unwrap();
    ssd1306.clear();
    ssd1306.draw().unwrap();

    let mut x = 1;
    let mut y = 5;
    let mut xadd: i16 = 1;
    let mut yadd: i16 = 1;

    loop {
        if x == 0 || x == 127 {
            xadd *= -1;
        }
        if y == 0 || y == 31 {
            yadd *= -1;
        }
        x += xadd;
        y += yadd;
        ssd1306.invert_pixel(x as u8, y as u8);
        ssd1306.draw().unwrap();
    }
}
