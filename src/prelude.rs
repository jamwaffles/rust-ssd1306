//! SSD1306 Prelude
 
use hal::blocking::i2c::Write;


/// Trait for writing data to SSD1306
pub trait Ssd1306Write {
    /// Error type
    type Error;

    /// Write a command to SSD1306
    fn write_cmd(&mut self, addr: u8, cmd: u8) -> Result<(), Self::Error>;
    /// Write data to SSD1306
    fn write_data(&mut self, addr: u8, data: &[u8]) -> Result<(), Self::Error>;
}

impl<I2C> Ssd1306Write for I2C
    where I2C: Write
{
    type Error = I2C::Error;

    fn write_cmd(&mut self, addr: u8, cmd: u8) -> Result<(), Self::Error> {
        let buf = [0, cmd];
        self.write(addr, &buf)
    }

    fn write_data(&mut self, addr: u8, data: &[u8]) -> Result<(), Self::Error> {
        let mut buf: [u8; 17] = [0; 17];
        buf[0] = 0x40;

        if data.len() == 0 {
            // error?
            return Ok(());
        }

        for chunk in data.chunks(16) {
            buf[1..].copy_from_slice(chunk);
            self.write(addr, &buf[..1+chunk.len()])?;
        }

        Ok(())
    }
}
