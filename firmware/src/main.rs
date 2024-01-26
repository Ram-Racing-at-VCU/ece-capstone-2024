#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed},
    interrupt, rcc, time,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        Channel, CountingMode,
    },
    Config,
};
use embassy_time::Timer;
use embedded_hal_02::Pwm;

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

    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let ch2n = ComplementaryPwmPin::new_ch2(p.PA12, OutputType::PushPull);
    let ch3 = PwmPin::new_ch3(p.PA10, OutputType::PushPull);
    let ch3n = ComplementaryPwmPin::new_ch3(p.PF0, OutputType::PushPull);

    let mut motor = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        Some(ch2),
        Some(ch2n),
        Some(ch3),
        Some(ch3n),
        None,
        None,
        time::hz(5), // 25 kHz / 2 (center aligned)
        CountingMode::CenterAlignedBothInterrupts,
    );

    info!("PWM Initialized!");

    motor.set_duty(Channel::Ch1, 0);
    motor.set_duty(Channel::Ch2, 0);
    motor.set_duty(Channel::Ch3, 0);

    motor.enable(Channel::Ch1);
    motor.enable(Channel::Ch2);
    motor.enable(Channel::Ch3);

    info!("Motor period: {}", motor.get_period());

    // servo.set_period(time::hz(60));

    // calculate duty cycle
    // let period = 1000.0 / (servo.get_period().0 as f32); // period in ms
    // let duty = 1.5; // duty in ms

    // let duty_frac = duty / period; // duty % (0..1)

    // set_duty_cycle(&mut servo, duty_frac, Channel::Ch2);
    // servo.enable(Channel::Ch2);

    let mut led = Output::new(p.PB8, Level::Low, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
        cortex_m::asm::wfi(); // sleep
    }
}

/// Set duty cycle given a fraction from 0 to 1 (inclusive)
fn set_duty_cycle<T>(servo: &mut T, frac: f32, channel: T::Channel)
where
    T: embedded_hal_02::Pwm,
    T::Duty: From<f32> + core::ops::Div<T::Duty, Output = T::Duty>,
{
    let div: T::Duty = (1.0 / frac).into();

    let max = servo.get_max_duty();

    let duty = max / div;

    servo.set_duty(channel, duty);
}
