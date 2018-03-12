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
use ssd1306::prelude::Write;
use ssd1306::{Ssd1306, ADDRESS, Resolution};
use ssd1306::cmd::{Command, Page};

fn draw_border<I2C>(i2c: &mut I2C) -> Result<(), I2C::Error>
where
    I2C: Write,
{
    Command::PageAddress(Page::Page0, Page::Page3).send(i2c, ADDRESS)?;
    Command::ColumnAddress(53, 74).send(i2c, ADDRESS)?;

    i2c.write_data(ADDRESS, &[0xE0])?;
    i2c.write_data(ADDRESS, &[0x20; 20])?;
    i2c.write_data(ADDRESS, &[0xE0])?;

    for _ in 0..2 {
        i2c.write_data(ADDRESS, &[0xFF])?;
        i2c.write_data(ADDRESS, &[0x00; 20])?;
        i2c.write_data(ADDRESS, &[0xFF])?;
    }

    i2c.write_data(ADDRESS, &[0x07])?;
    i2c.write_data(ADDRESS, &[0x04; 20])?;
    i2c.write_data(ADDRESS, &[0x07])?;

    Ok(())
}

fn draw_square<I2C>(i2c: &mut I2C, page: u8, col: u8, on: bool) -> Result<(), I2C::Error>
where
    I2C: Write,
{
    let page = match page {
        1 => Page::Page1,
        2 => Page::Page2,
        _ => panic!("Expected page 1 or 2"),
    };

    Command::PageAddress(page, page).send(i2c, ADDRESS)?;
    Command::ColumnAddress(col, col + 7).send(i2c, ADDRESS)?;
    if on {
        for _ in 0..8 {
            i2c.write_data(ADDRESS, &[0xFF])?;
        }
    } else {
        for _ in 0..8 {
            i2c.write_data(ADDRESS, &[0])?;
        }
    }
    Ok(())
}

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

    let mut i2c1 = ssd1306.free();

    draw_border(&mut i2c1).unwrap();

    let mut i = 0;
    loop {
        draw_square(&mut i2c1, 1 + (i % 2), 56 + (8 * (i / 2)), false).unwrap();
        i = (i + 1) % 4;
        draw_square(&mut i2c1, 1 + (i % 2), 56 + (8 * (i / 2)), true).unwrap();
        delay.delay_ms(500_u16);
    }
}
