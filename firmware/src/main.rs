#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::{cell::RefCell, f32::consts::PI};

use defmt::*;
use micromath::F32Ext;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    rcc,
    spi::{self, Spi},
    time,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        Channel, ComplementaryCaptureCompare16bitInstance, CountingMode,
    },
    usart::{self, UartRx},
    Config,
};

use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::{Instant, Timer};

// logger
use defmt_rtt as _;

// panic handler
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault()
}

// bind USART interrupt
bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    // configure all system clocks to 170 MHz
    let mut rcc = rcc::Config::default();
    // rcc.mux = rcc::ClockSrc::PLL;
    rcc.pll = Some(rcc::Pll {
        source: rcc::Pllsrc::HSI,       // 16 MHz
        prediv: rcc::PllPreDiv::DIV4,   // 16/4= 4 MHz
        mul: rcc::PllMul::MUL85,        // 4*85 = 340 MHz
        divp: Some(rcc::PllPDiv::DIV2), // 340/2 = 170 MHz
        divq: Some(rcc::PllQDiv::DIV2), // 340/2 = 170 MHz
        divr: Some(rcc::PllRDiv::DIV2), // 340/2 = 170 MHz
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
        time::khz(50),
        CountingMode::CenterAlignedBothInterrupts,
    );

    let mut spi_config = spi::Config::default();
    spi_config.frequency = time::khz(100);
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let _spi_device = SpiDevice::new(&spi, Output::new(p.PA11, Level::Low, Speed::VeryHigh));

    let uart_config = usart::Config::default();

    let _sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, uart_config);

    let _driver_cal = Output::new(p.PB7, Level::Low, Speed::Low);

    let _driver_enable = Output::new(p.PB6, Level::High, Speed::Low);

    let _adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);

    let _gate_driver_nfault = Input::new(p.PB0, Pull::Up);

    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);
    pwm.set_duty(Channel::Ch1, max / 2);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    let sin = generate_sin(1000.);

    loop {
        let x_n = map_range(sin(), (-1., 1.), (0., 1.));
        set_pwm_duty(&mut pwm, x_n, Channel::Ch1);
        Timer::after_micros(10).await;
    }
}

fn generate_sin(frequency: f32) -> impl Fn() -> f32 {
    move || {
        let t = Instant::now().as_micros() as f32 / 1e6;
        (2. * PI * frequency * t).sin()
    }
}

fn map_range(val: f32, before: (f32, f32), after: (f32, f32)) -> f32 {
    (val - before.0) * (after.1 - after.0) / (before.1 - before.0) + after.0
}

fn set_pwm_duty<T>(pwm: &mut ComplementaryPwm<T>, frac: f32, channel: Channel)
where
    T: ComplementaryCaptureCompare16bitInstance,
{
    let max = pwm.get_max_duty() as f32;
    let duty = (max * frac) as u16;
    pwm.set_duty(channel, duty);
}
