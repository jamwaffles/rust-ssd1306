//! Device-agnostic driver for SSD1306
//!
//! This driver was built using [`embedded_hal`] traits.
//!
//! [`embedded_hal`]: https://doc.rs/embedded-hal/~0.1
//!
//! # Examples
//!
//! ```
//! #![no_std]
//! #![deny(unsafe_code)]
//! #![deny(warnings)]
//!
//! extern crate f3;
//! extern crate cortex_m;
//! extern crate cortex_m_rt; // for abort-on-panic
//! extern crate embedded_hal;
//! extern crate ssd1306;
//!
//! use f3::hal::prelude::*;
//! use f3::hal::delay::Delay;
//! use f3::hal::i2c::I2c;
//! use f3::hal::stm32f30x;
//! use ssd1306::{Ssd1306, ADDRESS};
//!
//! fn main() {
//!     let cp = cortex_m::Peripherals::take().unwrap();
//!     let dp = stm32f30x::Peripherals::take().unwrap();
//!
//!     let mut flash = dp.FLASH.constrain();
//!     let mut rcc = dp.RCC.constrain();
//!     let clocks = rcc.cfgr.freeze(&mut flash.acr);
//!
//!     let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
//!     let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
//!     let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
//!     let i2c1 = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);
//!     let rst = gpiob.pb9.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
//!
//!     let delay = Delay::new(cp.SYST, clocks);
//!     let mut ssd1306 = Ssd1306::new(i2c1, ADDRESS, 128, 32);
//!
//!     ssd1306.reset(rst, delay);
//!     ssd1306.init().unwrap();
//!     ssd1306.clear();
//!     ssd1306.draw().unwrap();
//!
//!     let mut x = 1;
//!     let mut y = 5;
//!     let mut xadd: i16 = 1;
//!     let mut yadd: i16 = 1;
//!
//!     loop {
//!         if x == 0 || x == 127 {
//!             xadd *= -1;
//!         }
//!         if y == 0 || y == 31 {
//!             yadd *= -1;
//!         }
//!         x += xadd;
//!         y += yadd;
//!         ssd1306.invert_pixel(x as u8, y as u8);
//!         ssd1306.draw().unwrap();
//!     }
//! }
//! ```
#![no_std]
#![deny(missing_docs)]
#![deny(warnings)]

extern crate embedded_hal as hal;

/// SSD1306 Prelude
pub mod prelude;
/// Commands
pub mod cmd;

use hal::blocking::i2c;
use hal::digital::OutputPin;
use hal::blocking::delay::DelayMs;
use cmd::{AddrMode, Command, VcomhLevel};
use prelude::Write;

/// Default i2c address
pub const ADDRESS: u8 = 0x3C;
const BUF_SIZE: usize = 128 * 32 / 8;

/// Ssd1306
pub struct Ssd1306<I2C> {
    addr: u8,
    width: u8,
    height: u8,
    i2c: I2C,
    buf: [u8; BUF_SIZE],
}

impl<I2C> Ssd1306<I2C>
where
    I2C: i2c::Write,
{
    /// Create Ssd1306 object
    pub fn new(i2c: I2C, addr: u8, width: u8, height: u8) -> Ssd1306<I2C> {
        Ssd1306 {
            addr,
            i2c,
            width,
            height,
            buf: [0; BUF_SIZE],
        }
    }

    /// Release resources
    pub fn free(self) -> I2C {
        self.i2c
    }

    /// Reset display
    pub fn reset<RST, DELAY>(&mut self, mut rst: RST, mut delay: DELAY) -> (RST, DELAY)
    where
        RST: OutputPin,
        DELAY: DelayMs<u8>,
    {
        rst.set_high();
        delay.delay_ms(1);
        rst.set_low();
        delay.delay_ms(10);
        rst.set_high();
        (rst, delay)
    }

    /// Initialize display
    pub fn init(&mut self) -> Result<(), I2C::Error>
    {
        self.send_command(Command::DisplayOn(false))?;
        self.send_command(Command::DisplayClockDiv(0x8, 0x0))?;
        let mpx = self.height - 1;
        self.send_command(Command::Multiplex(mpx))?;
        self.send_command(Command::DisplayOffset(0))?;
        self.send_command(Command::StartLine(0))?;
        self.send_command(Command::ChargePump(true))?;
        self.send_command(Command::AddressMode(AddrMode::Horizontal))?;
        self.send_command(Command::SegmentRemap(true))?;
        self.send_command(Command::ReverseComDir(true))?;
        self.send_command(Command::ComPinConfig(false, false))?;
        self.send_command(Command::Contrast(0x8F))?;
        self.send_command(Command::PreChargePeriod(0x1, 0xF))?;
        self.send_command(Command::VcomhDeselect(VcomhLevel::Auto))?;
        self.send_command(Command::AllOn(false))?;
        self.send_command(Command::Invert(false))?;
        self.send_command(Command::EnableScroll(false))?;
        self.send_command(Command::DisplayOn(true))?;
        Ok(())
    }

    fn send_command(&mut self, cmd: Command) -> Result<(), I2C::Error> {
        cmd.send(&mut self.i2c, self.addr)
    }

    /// Clear output buffer
    pub fn clear(&mut self) {
        for i in 0..self.buf.len() {
            self.buf[i] = 0;
        }
    }

    /// Turn pixel off
    pub fn pixel_on(&mut self, x: u8, y: u8) {
        let b = &mut self.buf[((y as usize) / 8 * 128) + (x as usize)];
        *b |= 1 << (y % 8);
    }

    /// Turn pixel on
    pub fn pixel_off(&mut self, x: u8, y: u8) {
        let b = &mut self.buf[((y as usize) / 8 * 128) + (x as usize)];
        *b &= !(1 << (y % 8));
    }

    /// Swap pixel value
    pub fn invert_pixel(&mut self, x: u8, y: u8) {
        let byte = &mut self.buf[((y as usize) / 8 * 128) + (x as usize)];
        let bit = 1 << (y % 8);
        if *byte & bit == 0 {
            *byte |= bit;
        } else {
            *byte &= !bit;
        }
    }

    /// Draw buffer to display
    pub fn draw(&mut self) -> Result<(), I2C::Error> {
        let ecol = self.width - 1;
        let epage = self.height - 1;
        self.send_command(Command::ColumnAddress(0, ecol))?;
        self.send_command(Command::PageAddress(0.into(), epage.into()))?;
        self.i2c.write_data(self.addr, &self.buf)?;
        Ok(())
    }
}
