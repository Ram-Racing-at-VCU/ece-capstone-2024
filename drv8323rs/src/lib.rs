//! Driver for the DRV8323RS chip. Also works for similar chips, see the datasheet for details.
//!
//! This crate simply controls the SPI bus and implements register definitions provided in the datasheet.

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[cfg(all(test, not(target_arch = "arm")))]
#[macro_use]
extern crate std;

#[cfg(feature = "async")]
mod async_impl;

#[cfg(feature = "async")]
pub use device_register_async::{EditRegister, ReadRegister, WriteRegister};

#[cfg(not(feature = "async"))]
mod blocking_impl;

#[cfg(not(feature = "async"))]
pub use device_register::{EditRegister, ReadRegister, WriteRegister};

pub mod registers;

/// The DRV8323RS. Use `.read()`, `.write()` and `.edit()` functions to access its registers.
pub struct Drv8323rs<T> {
    /// The 4-wire SPI bus
    spi: T,
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

#[cfg(all(test, not(target_arch = "arm"), feature = "async"))]
mod test {
    use crate::registers::{DriveControl, PwmMode};

    #[test]
    fn test_bit_pattern() {
        let control = Control::new().with_pwm_mode(PwmMode::_3x);

        let bits = control.into_bytes();

        println!("bits: {:?}", bits);

        assert_eq!(bits, [0b0000_0000, 0b0010_0000])
    }
}
