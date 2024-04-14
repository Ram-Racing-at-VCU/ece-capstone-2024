use embassy_stm32::time::{mhz, Hertz};

/// How many reads are done to the DRV to ensure that
/// the bus is operating properly.
pub const DRV_INIT_READS: usize = 10_000;

/// SPI Frequency
pub const SPI_FREQUENCY: Hertz = mhz(10);
