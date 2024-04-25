use embassy_stm32::time::{khz, mhz, Hertz};

/// How many reads are done to the DRV to ensure that
/// the bus is operating properly.
pub const DRV_INIT_READS: usize = 10_000;

/// SPI Frequency
pub const SPI_FREQUENCY: Hertz = mhz(5);

/// PWM Frequency
pub const PWM_FREQUENCY: Hertz = khz(45);

/// Angle filter cutoff frequency in hz
pub const F_C: i32 = 850;

/// Angle filter sampling frequency in khz
pub const F_S: i32 = 17;

// /// Speed filter window size
// pub const WINDOW_SIZE: usize = 31;

/// Motor Phase Resistance
pub const RESISTANCE: f32 = 6.2832e-3 + 2.5e-3 + 0.85e-3;

/// Motor Phase Inductance
pub const INDUCTANCE: f32 = 2.56e-6;

/// PI Controller Bandwidth
pub const BANDWIDTH: f32 = 10000.0;
