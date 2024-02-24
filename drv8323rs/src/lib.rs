use device_register::{Register, RegisterInterface};
use embedded_hal::spi::SpiDevice;

pub mod registers;

struct Drv8323rs<T>
where
    T: SpiDevice<u8>,
{
    spi: T,
}

enum Error<E1, E2> {
    Spi(E1),
    Serialization(E2),
}

trait SerializableRegister<const SIZE: usize>
where
    Self: Sized,
{
    type Error;
    fn from_bytes(bytes: [u8; SIZE]) -> Result<Self, Self::Error>;
    fn to_bytes(self) -> Result<[u8; SIZE], Self::Error>;
}

impl<R, Spi> RegisterInterface<R, u8> for Drv8323rs<Spi>
where
    R: Register<Address = u8> + SerializableRegister<2> + Copy,
    Spi: SpiDevice,
{
    type Error = Error<Spi::Error, R::Error>;

    fn read_register(&mut self) -> Result<R, Self::Error> {
        // issue read command
        self.spi
            .write(&[0x80 | (R::ADDRESS << 3), 0])
            .map_err(Error::Spi)?;

        // fill buffer
        let mut buf = [0, 0];
        self.spi.read(&mut buf).map_err(Error::Spi)?;

        // deserialize
        R::from_bytes(buf).map_err(Error::Serialization)
    }

    fn write_register(&mut self, register: &R) -> Result<(), Self::Error> {
        // serialize
        let mut buf = register.to_bytes().map_err(Error::Serialization)?;

        // add address
        buf[0] |= R::ADDRESS << 3;

        // issue write command
        self.spi.write(&buf).map_err(Error::Spi)
    }
}

#[cfg(test)]
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
