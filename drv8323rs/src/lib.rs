//! Driver for the DRV8323RS chip. Also works for similar chips, see the datasheet for details.
//!
//! This crate simply controls the SPI bus and implements register definitions provided in the datasheet.

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[cfg(all(test, not(target_arch = "arm")))]
#[macro_use]
extern crate std;

use device_register::{Register, RegisterInterface};
use embedded_hal::spi::{Operation, SpiDevice};

pub use device_register::{EditRegister, ReadRegister, WriteRegister};

pub mod registers;

/// The DRV8323RS. Use `.read()`, `.write()` and `.edit()` functions to access its registers.
pub struct Drv8323rs<T>
where
    T: SpiDevice<u8>,
{
    /// The 4-wire SPI bus
    spi: T,
}

impl<T: SpiDevice<u8>> Drv8323rs<T> {
    /// Constructor method
    pub fn new(spi: T) -> Self {
        Self { spi }
    }
}

/// Internal trait used for `RegisterInterface<R, A>` implementation
trait SerializableRegister<const SIZE: usize>
where
    Self: Sized,
{
    /// Convert a buffer of bytes into self
    fn from_bytes(bytes: [u8; SIZE]) -> Self;
    /// Convert self into a buffer of bytes
    fn to_bytes(self) -> [u8; SIZE];
}

impl<R, Spi> RegisterInterface<R, u8> for Drv8323rs<Spi>
where
    R: Register<Address = u8> + SerializableRegister<2> + Copy,
    Spi: SpiDevice,
{
    type Error = Spi::Error;

    fn read_register(&mut self) -> Result<R, Self::Error> {
        let mut buf = [0, 0];

        self.spi.transaction(&mut [
            Operation::Write(&[0x80 | (R::ADDRESS << 3), 0]),
            Operation::Read(&mut buf),
        ])?;

        // deserialize
        Ok(R::from_bytes(buf))
    }

    fn write_register(&mut self, register: &R) -> Result<(), Self::Error> {
        // serialize
        let mut buf = register.to_bytes();

        // add address
        buf[0] |= R::ADDRESS << 3;

        // issue write command
        self.spi.write(&buf)
    }
}

#[cfg(all(test, not(target_arch = "arm")))]
mod test {

    use crate::registers::{Control, PwmMode};

    #[test]
    fn test_bit_pattern() {
        let control = Control::new().with_pwm_mode(PwmMode::_3x);

        let bits = control.into_bytes();

        println!("bits: {:?}", bits);

        assert_eq!(bits, [0b0000_0000, 0b0010_0000])
    }
}
