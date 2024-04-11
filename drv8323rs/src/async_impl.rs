//! Asynchronous version of the register traits

use device_register::Register;
use device_register_async::RegisterInterface;
use embedded_hal_async::spi::SpiDevice;

use crate::{Drv8323rs, SerializableRegister};

impl<T: SpiDevice<u8>> Drv8323rs<T> {
    /// Async version of the constructor
    pub fn new(spi: T) -> Self {
        Self { spi }
    }
}

impl<R, Spi> RegisterInterface<R, u8> for Drv8323rs<Spi>
where
    R: Register<Address = u8> + SerializableRegister<2> + Copy,
    Spi: SpiDevice,
{
    type Error = Spi::Error;

    async fn read_register(&mut self) -> Result<R, Self::Error> {
        let mut buf = [0x80 | (R::ADDRESS << 3), 0];

        /* Half-duplex communication */
        // self.spi
        //     .transaction(&mut [
        //         Operation::Write(&[0x80 | (R::ADDRESS << 3), 0]),
        //         Operation::Read(&mut buf),
        //     ])
        //     .await?;

        /* Full-duplex communication */
        self.spi.transfer_in_place(&mut buf).await?;

        // deserialize
        Ok(R::from_bytes(buf))
    }

    async fn write_register(&mut self, register: &R) -> Result<(), Self::Error> {
        // serialize
        let mut buf = register.to_bytes();

        // add address
        buf[0] |= R::ADDRESS << 3;

        // issue write command
        self.spi.write(&buf).await
    }
}
