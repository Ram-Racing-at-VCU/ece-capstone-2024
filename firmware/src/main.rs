#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::OutputType,
    rcc, time,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    Config,
};

use {defmt_rtt as _, panic_probe as _}; // logger & panic handler

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let mut config = Config::default();

    // configure all system clocks to 170 MHz
    config.rcc = rcc::Config {
        mux: rcc::ClockSrc::PLL,
        pll: Some(rcc::Pll {
            source: rcc::PllSrc::HSI,     // 16 MHz
            prediv_m: rcc::PllM::DIV4,    // 16/4= 4 MHz
            mul_n: rcc::PllN::MUL85,      // 4*85 = 340 MHz
            div_p: Some(rcc::PllP::DIV2), // 340/2 = 170 MHz
            div_q: Some(rcc::PllQ::DIV2), // 340/2 = 170 MHz
            div_r: Some(rcc::PllR::DIV2), // 340/2 = 170 MHz
        }),
        ..Default::default()
    };

    let p = embassy_stm32::init(config);

    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let mut servo = SimplePwm::new(
        p.TIM1,
        None,
        Some(ch2),
        None,
        None,
        time::mhz(1),
        CountingMode::EdgeAlignedUp,
    );

    info!("PWM Initialized!");

    servo.set_period(time::hz(60));

    // calculate duty cycle
    let period = 1000.0 / (servo.get_period().0 as f32); // period in ms
    let duty = 1.5; // duty in ms

    let duty_frac = duty / period; // duty % (0..1)
    let div = (1.0 / duty_frac) as u16; // divider

    let max = servo.get_max_duty();
    let duty = max / div;

    servo.set_duty(Channel::Ch2, duty);

    servo.enable(Channel::Ch2);

    loop {
        cortex_m::asm::wfi(); // sleep
    }
}
