//! LTC1408-12 Driver

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use embedded_hal_async::spi::SpiBus;

/// The LTC1408-12
pub struct Ltc1408_12<T> {
    /// SPI bus
    spi: T,
    /// number of enabled channels
    channels: usize,
}

impl<T: SpiBus> Ltc1408_12<T> {
    /// Constructor for the LTC1408-12. Input the number of channels enabled (1..=6)
    pub fn new(spi: T, channels: usize) -> Option<Self> {
        if channels > 0 && channels <= 6 {
            Some(Self { spi, channels })
        } else {
            None
        }
    }

    /// Bit shifting to get actual value
    #[inline]
    fn buf_to_ch(buf0: u8, buf1: u8) -> u16 {
        let w1 = (buf0 as u16) & 0x3F;
        let w2 = (buf1 as u16) & 0xFC;

        (w1 << (8 - 2)) | (w2 >> 2)
    }

    /// Read the enabled channels from the LTC. Note: disabled channels return 0.
    pub async fn read(&mut self) -> Result<[u16; 6], T::Error> {
        let mut buf = [0u8; 12];

        self.spi.read(&mut buf[0..self.channels * 2]).await?;

        Ok([
            Self::buf_to_ch(buf[0], buf[1]),
            Self::buf_to_ch(buf[2], buf[3]),
            Self::buf_to_ch(buf[4], buf[5]),
            Self::buf_to_ch(buf[6], buf[7]),
            Self::buf_to_ch(buf[8], buf[9]),
            Self::buf_to_ch(buf[10], buf[11]),
        ])
    }
}
