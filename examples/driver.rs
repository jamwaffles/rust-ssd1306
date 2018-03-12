#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]

extern crate cortex_m;
extern crate cortex_m_rt; // for abort-on-panic
extern crate embedded_hal;
extern crate f3;
extern crate ssd1306;

use f3::hal::prelude::*;
use f3::hal::delay::Delay;
use f3::hal::i2c::I2c;
use f3::hal::stm32f30x;
use ssd1306::{Ssd1306, ADDRESS, Resolution};

fn main() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c1 = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);
    let mut rst = gpiob
        .pb9
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut ssd1306 = Ssd1306::new(i2c1, ADDRESS, Resolution::R128x32, true);

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
