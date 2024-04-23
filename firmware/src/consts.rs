use embassy_stm32::time::{khz, mhz, Hertz};

/// How many reads are done to the DRV to ensure that
/// the bus is operating properly.
pub const DRV_INIT_READS: usize = 10_000;

/// SPI Frequency
pub const SPI_FREQUENCY: Hertz = mhz(5);

/// PWM Frequency
pub const PWM_FREQUENCY: Hertz = khz(45);

/// Filter cutoff
pub const CUTOFF: f32 = 100.0;

/// Filter sample rate
pub const SAMPLE_RATE: f32 = 28.5e3;

/// Filter sensitivity factor
pub const SENSITIVITY: f32 = 0.01;
