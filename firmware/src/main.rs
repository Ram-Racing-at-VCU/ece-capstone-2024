#![no_std]
#![no_main]

mod helpers;

use cortex_m::prelude::_embedded_hal_Pwm;
use drv8323rs::Drv8323rs;
use sbus::Sbus;

use defmt::*;

use control_algorithms::foc::*;
use control_algorithms::svpwm::*;

use core::f32::consts::PI;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::{task, Spawner};
use embassy_stm32::{
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{self, DMA1_CH3, USART2},
    rcc,
    spi::{self, Spi},
    time,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        Channel, CountingMode,
    },
    usart::{self, DataBits, Parity, StopBits, UartRx},
    Config,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;

// bind logger
use defmt_rtt as _;

// bind panic handler
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault()
}

#[embassy_executor::main]

// bind USART interrupt
// embassy_stm32::bind_interrupts!(struct Irqs {
//     USART2 => usart::InterruptHandler<peripherals::USART2>;
// });
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // configure all system clocks to 170 MHz
    let mut rcc = rcc::Config::default();
    // rcc.mux = rcc::ClockSrc::PLL;
    rcc.pll = Some(rcc::Pll {
        source: rcc::PllSource::HSI,  // 16 MHz
        prediv_m: rcc::PllM::DIV4,    // 16/4= 4 MHz
        mul_n: rcc::PllN::MUL85,      // 4*85 = 340 MHz
        div_p: Some(rcc::PllP::DIV2), // 340/2 = 170 MHz
        div_q: Some(rcc::PllQ::DIV2), // 340/2 = 170 MHz
        div_r: Some(rcc::PllR::DIV2), // 340/2 = 170 MHz
    });
    config.rcc = rcc;

    let p = embassy_stm32::init(config);

    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let ch2n = ComplementaryPwmPin::new_ch2(p.PA12, OutputType::PushPull);
    let ch3 = PwmPin::new_ch3(p.PA10, OutputType::PushPull);
    let ch3n = ComplementaryPwmPin::new_ch3(p.PF0, OutputType::PushPull);

    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        Some(ch2),
        Some(ch2n),
        Some(ch3),
        Some(ch3n),
        None,
        None,
        time::khz(25),
        CountingMode::CenterAlignedBothInterrupts,
    );

    let mut spi_config = spi::Config::default();
    spi_config.frequency = time::mhz(10);
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi: Mutex<NoopRawMutex, _> = Mutex::new(spi);

    let drv8323rs = SpiDevice::new(&spi, Output::new(p.PA11, Level::Low, Speed::VeryHigh));
    let mut drv8323rs = Drv8323rs::new(drv8323rs);
    helpers::driver_setup(&mut drv8323rs).await;

    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 100_000;
    uart_config.data_bits = DataBits::DataBits8;
    uart_config.stop_bits = StopBits::STOP2;
    uart_config.parity = Parity::ParityEven;
    uart_config.invert_rx = true;

    // let sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, uart_config).unwrap();

    // let sbus = helpers::usart_to_sbus(sbus);

    // spawner.spawn(get_receiver_data(sbus)).unwrap();

    let _driver_cal = Output::new(p.PB7, Level::Low, Speed::Low);

    let mut _driver_enable = Output::new(p.PB6, Level::High, Speed::Low);

    let _adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);

    let _gate_driver_nfault = Input::new(p.PB0, Pull::Up);

    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);
    // pwm.set_duty(Channel::Ch1, max / 2);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    // let sin = helpers::generate_sin(1000.);
    let frequency: f32 = 1000.;
    let phase_angle = helpers::generate_angle(frequency);
    let v_a = helpers::generate_cos(frequency, 0.);
    let v_b = helpers::generate_cos(frequency, -2. * PI / 3.);
    let v_c = helpers::generate_cos(frequency, 2. * PI / 3.);

    loop {
        // let v_abc = na::SVector::<f32, 3>::new(12. * va(), 12. * vb(), 12. * vc());
        // let v_clarke = clarke_transform(v_abc);

        let (v_d, v_q) = dq_transform(10. * v_a(), 10. * v_b(), 10. * v_c(), phase_angle());
        let (v_alpha, v_beta) = inverse_park_transform(v_d, v_q, phase_angle());
        // let v_park = park_transform(v_clarke, phase_angle());
        let (t_a, t_b, t_c) = svpwm(v_alpha, v_beta, phase_angle(), 12., 4e-5);
        _driver_enable.set_high();
        _driver_enable.set_low();
        // pwm.set_duty(Channel::Ch1, (t_a / 2e-5) as u16);
        helpers::set_pwm_duty(&mut pwm, t_a / 4e-5, Channel::Ch1);
        // info!("Duty Cycle: {}", t_a / 1e-5);

        // info!(
        //     "t_a: {}, t_b: {}, t_c: {}, Max duty cycle period: {}",
        //     t_a,
        //     t_b,
        //     t_c,
        //     pwm.get_max_duty() as f32
        // );
        // let v_clarke_valid = inverse_park_transform(v_park, phase_angle);

        //let x_n = helpers::map_range(sin(), (-1., 1.), (0., 1.));
        //helpers::set_pwm_duty(&mut pwm, x_n, Channel::Ch1);
        // Timer::after_micros(10).await;
    }
}

// #[task]
// async fn get_receiver_data(mut sbus: Sbus<helpers::UartRxWrapper<'static, USART2, DMA1_CH3>>) {
//     loop {
//         match sbus.get_packet().await {
//             Ok(data) => {
//                 // debug!("raw: {:#04x}", data.into_bytes());
//                 debug!(
//                     "ch1: {:05}, ch2: {:05}, ch3: {:05}, ch4: {:05}",
//                     data.ch1(),
//                     data.ch2(),
//                     data.ch3(),
//                     data.ch4(),
//                 );
//             }
//             Err(e) => warn!("error happened while reading sbus: {}", e),
//         }
//     }
// }
