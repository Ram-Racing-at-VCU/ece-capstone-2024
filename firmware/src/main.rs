#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::OutputType,
    rcc::{self, ClockSrc},
    time::{hz, khz},
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    Config,
};

use {defmt_rtt as _, panic_probe as _}; // logger & panic handler

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let mut servo = SimplePwm::new(
        p.TIM1,
        None,
        Some(ch2),
        None,
        None,
        khz(10),
        CountingMode::EdgeAlignedUp,
    );

    servo.enable(Channel::Ch2);

    info!("PWM Initialized!");

    servo.set_period(hz(50));
    servo.set_duty(Channel::Ch2, 4500);

    loop {
        // cortex_m::asm::wfi(); // sleep
    }
}
