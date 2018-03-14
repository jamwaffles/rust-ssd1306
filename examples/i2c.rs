#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt; // for abort-on-panic
extern crate embedded_hal;
extern crate ssd1306;
extern crate stm32f103xx_hal as blue_pill;

use blue_pill::prelude::*;
use blue_pill::i2c::{DutyCycle, I2c, Mode};
// use blue_pill::delay::Delay;
use ssd1306::{Resolution, Ssd1306, ADDRESS};

fn main() {
    // let cp = cortex_m::Peripherals::take().unwrap();
    let dp = blue_pill::stm32f103xx::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = I2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio1to1,
        },
        clocks,
        &mut rcc.apb1,
    );

    // let mut delay = Delay::new(cp.SYST, clocks);
    let mut ssd1306 = Ssd1306::new(i2c, ADDRESS, Resolution::R128x64, true);

    // ssd1306.reset(&mut rst, &mut delay);
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
