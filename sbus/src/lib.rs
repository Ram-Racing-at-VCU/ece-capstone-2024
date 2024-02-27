//! SBUS packet decoder, using UART to retrieve data

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use embedded_io_async::{Read, ReadExactError};
use modular_bitfield::prelude::*;

/// Futaba SBUS driver using asynchronous USART bus
pub struct Sbus<T: Read> {
    /// actual USART pin to read from
    bus: T,
}

/// Error types that can occur during reading of the USART pin
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    /// An error occurred while reading from the bus
    ReadError(T),
    /// An error occurred while reading from the bus
    ReadExactError(ReadExactError<T>),
    /// The first byte was not the frame sync byte
    FrameSync,
}

impl<T> From<ReadExactError<T>> for Error<T> {
    fn from(value: ReadExactError<T>) -> Self {
        Self::ReadExactError(value)
    }
}

impl<T> From<T> for Error<T> {
    fn from(value: T) -> Self {
        Self::ReadError(value)
    }
}

/// Data from the Futaba SBUS
#[bitfield(bytes = 25)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct Data {
    #[skip]
    start_byte: u8,
    pub ch1: B11,
    pub ch2: B11,
    pub ch3: B11,
    pub ch4: B11,
    pub ch5: B11,
    pub ch6: B11,
    pub ch7: B11,
    pub ch8: B11,
    pub ch9: B11,
    pub ch10: B11,
    pub ch11: B11,
    pub ch12: B11,
    pub ch13: B11,
    pub ch14: B11,
    pub ch15: B11,
    pub ch16: B11,
    pub dig_ch1: bool,
    pub dig_ch2: bool,
    /// Equivalent to the red LED on the receiver
    pub frame_lost: bool,
    pub failsafe_activated: bool,
    #[skip]
    __: B4,
    #[skip]
    stop_byte: u8,
}

impl<T: Read> Sbus<T> {
    /// Constructor for driver
    pub fn new(bus: T) -> Self {
        Self { bus }
    }

    /// Gets a packet, waiting for the correct number of bytes
    pub async fn get_packet(&mut self) -> Result<Data, Error<T::Error>> {
        let mut buf = [0; 25];

        self.bus.read_exact(&mut buf).await?;

        if buf[0] != 0x0F || buf[24] != 0x00 {
            if let Some((idx, _)) = buf.iter().enumerate().find(|(_, i)| **i == 0xF0) {
                // attempt to sync the buffer
                self.bus.read_exact(&mut buf[0..idx]).await?;
            }

            return Err(Error::FrameSync);
        }

        Ok(Data::from_bytes(buf))
    }
}
