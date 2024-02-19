// #![no_std]
// #![no_main]
// #![feature(type_alias_impl_trait)]

// use defmt::info;
// use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
// use embassy_executor::Spawner;
// use embassy_stm32::{
//     bind_interrupts,
//     gpio::{Input, Level, Output, OutputType, Pull, Speed},
//     rcc,
//     spi::{self, Spi},
//     time,
//     timer::{
//         complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
//         simple_pwm::PwmPin,
//         Channel, CountingMode,
//     },
//     usart::{self, UartRx},
//     Config,
// };
// use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
// use embassy_time::Timer;
// use embedded_hal_02::Pwm;

// use {defmt_rtt as _, panic_probe as _}; // logger & panic handler

// bind_interrupts!(struct Irqs {
//     USART2 => usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
// });

// #[embassy_executor::main]
// async fn main(_s: Spawner) {
//     let mut config = Config::default();

//     // configure all system clocks to 170 MHz
//     config.rcc = rcc::Config {
//         mux: rcc::ClockSrc::PLL,
//         pll: Some(rcc::Pll {
//             source: rcc::PllSrc::HSI,     // 16 MHz
//             prediv_m: rcc::PllM::DIV4,    // 16/4= 4 MHz
//             mul_n: rcc::PllN::MUL85,      // 4*85 = 340 MHz
//             div_p: Some(rcc::PllP::DIV2), // 340/2 = 170 MHz
//             div_q: Some(rcc::PllQ::DIV2), // 340/2 = 170 MHz
//             div_r: Some(rcc::PllR::DIV2), // 340/2 = 170 MHz
//         }),
//         ..Default::default()
//     };

//     let p = embassy_stm32::init(config);

//     let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
//     let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
//     let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
//     let ch2n = ComplementaryPwmPin::new_ch2(p.PA12, OutputType::PushPull);
//     let ch3 = PwmPin::new_ch3(p.PA10, OutputType::PushPull);
//     let ch3n = ComplementaryPwmPin::new_ch3(p.PF0, OutputType::PushPull);

//     let mut motor = ComplementaryPwm::new(
//         p.TIM1,
//         Some(ch1),
//         Some(ch1n),
//         Some(ch2),
//         Some(ch2n),
//         Some(ch3),
//         Some(ch3n),
//         None,
//         None,
//         time::hz(5), // 25 kHz / 2 (center aligned)
//         CountingMode::CenterAlignedBothInterrupts,
//     );

//     info!("PWM Initialized!");

//     motor.set_duty(Channel::Ch1, 0);
//     motor.set_duty(Channel::Ch2, 0);
//     motor.set_duty(Channel::Ch3, 0);

//     motor.enable(Channel::Ch1);
//     motor.enable(Channel::Ch2);
//     motor.enable(Channel::Ch3);

//     info!("Motor period: {}", motor.get_period());

//     // servo.set_period(time::hz(60));

//     // calculate duty cycle
//     // let period = 1000.0 / (servo.get_period().0 as f32); // period in ms
//     // let duty = 1.5; // duty in ms

//     // let duty_frac = duty / period; // duty % (0..1)

//     // set_duty_cycle(&mut servo, duty_frac, Channel::Ch2);
//     // servo.enable(Channel::Ch2);

//     let mut led = Output::new(p.PB8, Level::Low, Speed::Low);

//     let mut spi_config = spi::Config::default();
//     spi_config.frequency = time::khz(100);
//     spi_config.mode = spi::MODE_1;

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::f32::consts::PI;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    rcc,
    spi::{self, Spi},
    time,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::{PwmPin, SimplePwm},
        CaptureCompare16bitInstance, Channel, ComplementaryCaptureCompare16bitInstance,
        CountingMode,
    },
    usart::{self, UartRx},
    Config,
};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Instant;
use embassy_time::Timer;
use embedded_hal_02::Pwm;
use micromath::F32Ext;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        None,
        None,
        None,
        None,
        None,
        None,
        time::hz(50_000),
        CountingMode::CenterAlignedBothInterrupts,
    );

    // let mut pwm = ComplementaryPwm::new(
    //     p.TIM1,
    //     Some(ch1),
    //     Some(ch1n),
    //     None,
    //     None,
    //     None,
    //     None,
    //     None,
    //     None,
    //     time::hz(10_000),
    //     CountingMode::CenterAlignedBothInterrupts,
    // );

    let mut spi_config = spi::Config::default();
    spi_config.frequency = time::khz(100);
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi: Mutex<NoopRawMutex, _> = Mutex::new(spi);

    let spi_device = SpiDevice::new(&spi, Output::new(p.PA11, Level::Low, Speed::VeryHigh));

    let mut uart_config = usart::Config::default();

    let sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, uart_config);

    let driver_cal = Output::new(p.PB7, Level::Low, Speed::Low);

    let driver_enable = Output::new(p.PB6, Level::High, Speed::Low);

    let adc = Output::new(p.PA2, Level::Low, Speed::Low);

    let gate_driver = Input::new(p.PB0, Pull::Up);

    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);
    pwm.set_duty(Channel::Ch1, max / 2);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    generate_sin(&mut pwm, 1000.).await;
}

// (f32::sin(Instant::now().as_micros() as f32) + 1.0f32) * 0.5f32

// Set duty cycle given a fraction from 0 to 1 (inclusive)
fn set_duty_cycle<T>(servo: &mut T, frac: f32, channel: T::Channel)
where
    T: embedded_hal_02::Pwm<Duty = u16>,
{
    let div = 1.0 / frac;

    let max = servo.get_max_duty() as f32;

    let duty = (max / div) as u16;

    servo.set_duty(channel, duty);
}

async fn generate_sin<'a, T>(pwm: &'a mut ComplementaryPwm<'a, T>, f: f32)
where
    T: ComplementaryCaptureCompare16bitInstance,
{
    loop {
        let t = (Instant::now().as_micros()) as f32 / 1e6;
        let frac = (f32::sin(2. * PI * f * t) + 1.0f32) * 0.5f32;
        // info!("frac is {}", frac);
        set_duty_cycle(pwm, frac, Channel::Ch1);
        Timer::after_micros(10).await;
    }
}
