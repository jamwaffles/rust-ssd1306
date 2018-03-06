//! Device-agnostic driver for SSD1306
#![no_std]
#![deny(missing_docs)]
#![deny(warnings)]

extern crate embedded_hal as hal;

/// SSD1306 Prelude
pub mod prelude;
/// Commands
pub mod cmd;

use hal::blocking::i2c::Write;
use hal::digital::OutputPin;
use hal::blocking::delay::DelayMs;
use cmd::{AddrMode, Command, VcomhLevel};
use prelude::Ssd1306Write;

/// Default i2c address
pub const ADDRESS: u8 = 0x3C;
const BUF_SIZE: usize = 128 * 32 / 8;

/// Ssd1306
pub struct Ssd1306<I2C, RST> {
    addr: u8,
    width: u8,
    height: u8,
    i2c: I2C,
    rst: RST,
    buf: [u8; BUF_SIZE],
}

impl<I2C, RST> Ssd1306<I2C, RST>
where
    I2C: Write,
    RST: OutputPin,
{
    /// Create Ssd1306 object
    pub fn new(i2c: I2C, addr: u8, rst: RST, width: u8, height: u8) -> Ssd1306<I2C, RST> {
        Ssd1306 {
            addr: addr,
            i2c: i2c,
            rst: rst,
            width: width,
            height: height,
            buf: [0; BUF_SIZE],
        }
    }

    /// Release resources
    pub fn free(self) -> (I2C, RST) {
        (self.i2c, self.rst)
    }

    /// Reset display
    pub fn reset<DELAY>(&mut self, mut delay: DELAY) -> DELAY
    where
        DELAY: DelayMs<u8>,
    {
        self.rst.set_high();
        delay.delay_ms(1);
        self.rst.set_low();
        delay.delay_ms(10);
        self.rst.set_high();
        delay
    }

    /// Initialize display
    pub fn init<DELAY>(&mut self, delay: DELAY) -> Result<DELAY, I2C::Error>
    where
        DELAY: DelayMs<u8>,
    {
        let delay = self.reset(delay);
        self.setup_display()?;
        Ok(delay)
    }

    fn setup_display(&mut self) -> Result<(), I2C::Error> {
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
        let b = &mut self.buf[((y as usize) / 8 * 128) + (x as usize)];
        if *b | (y % 8) == 0 {
            *b |= 1 << (y % 8);
        } else {
            *b &= !(1 << (y % 8));
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
