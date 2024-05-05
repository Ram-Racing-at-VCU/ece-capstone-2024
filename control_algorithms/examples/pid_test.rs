#![no_std]
#![no_main]

use control_algorithms::pid::PIDController;

use defmt::*;

use defmt_rtt as _;

#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault()
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut controller = PIDController::new(1., 1., 1., None);

    let input_signal = [1f32; 1000];

    let measurement: f32 = 0.;

    let mut output_signal = [0f32; 1000];

    for (i, input) in input_signal.iter().enumerate() {
        output_signal[i] = controller.output(*input, measurement, 0.001);
    }

    for i in output_signal {
        println!("{}", i)
    }

    loop {
        cortex_m::asm::wfe();
    }
}
