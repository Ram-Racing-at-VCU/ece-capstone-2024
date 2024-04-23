//! LTC1408-12 Driver

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/* SPI bus/device override */

#[cfg(not(feature = "unsafe_spi_device"))]
use embedded_hal_async::spi::SpiBus as Spi;
#[cfg(feature = "unsafe_spi_device")]
use embedded_hal_async::spi::SpiDevice as Spi;

/// The LTC1408-12
pub struct Ltc1408_12<T: Spi> {
    /// SPI bus
    spi: T,
    /// number of enabled channels
    channels: usize,
}

impl<T: Spi> Ltc1408_12<T> {
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

    /// Conversion from u16 to f32
    #[inline]
    fn ch_to_float(input: u16) -> f32 {
        (input as f32) * (2.5 / 4096.0)
    }

    /// Read the enabled channels from the LTC. Note: disabled channels return 0.
    pub async fn read(&mut self) -> Result<[f32; 6], T::Error> {
        let mut buf = [0u8; 12];

        self.spi.read(&mut buf[0..self.channels * 2]).await?;

        Ok([
            Self::ch_to_float(Self::buf_to_ch(buf[0], buf[1])),
            Self::ch_to_float(Self::buf_to_ch(buf[2], buf[3])),
            Self::ch_to_float(Self::buf_to_ch(buf[4], buf[5])),
            Self::ch_to_float(Self::buf_to_ch(buf[6], buf[7])),
            Self::ch_to_float(Self::buf_to_ch(buf[8], buf[9])),
            Self::ch_to_float(Self::buf_to_ch(buf[10], buf[11])),
        ])
    }
}
