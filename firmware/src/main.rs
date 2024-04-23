#![no_std]
#![no_main]

mod consts;
mod driver;
mod helpers;

use core::f32::consts::PI;

use consts::{CUTOFF, PWM_FREQUENCY, SAMPLE_RATE, SENSITIVITY, SPI_FREQUENCY};
use driver::{check_driver, report_status, setup_driver};

use drv8323rs::Drv8323rs;
use dyn_smooth::DynamicSmootherEcoF32;
use ltc1408_12::Ltc1408_12;
use sbus::Sbus;

use defmt::{assert, info};
use micromath::F32Ext;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::{task, Spawner};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, OutputType, Pull, Speed},
    mode::Async,
    peripherals, rcc,
    spi::{self, Spi},
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        low_level::CountingMode,
        simple_pwm::PwmPin,
        Channel,
    },
    usart::{self, DataBits, Parity, StopBits, UartRx},
    Config,
};
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
    once_lock::OnceLock,
};
use embassy_time::{Instant, Timer};

// bind logger
use defmt_rtt as _;

// bind panic handler
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault()
}

// bind USART interrupt
embassy_stm32::bind_interrupts!(
    struct Irqs {
        USART2 => usart::InterruptHandler<peripherals::USART2>;
    }
);

// shared resources

static SPI_BUS: OnceLock<Mutex<NoopRawMutex, Spi<'static, peripherals::SPI1, Async>>> =
    OnceLock::new();

static ENABLE: Mutex<ThreadModeRawMutex, bool> = Mutex::new(false);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // configure all system clocks to 170 MHz
    let mut rcc = rcc::Config::default();
    rcc.sys = rcc::Sysclk::PLL1_R;
    rcc.pll = Some(rcc::Pll {
        source: rcc::PllSource::HSI,    // 16 MHz
        prediv: rcc::PllPreDiv::DIV4,   // 16/4= 4 MHz
        mul: rcc::PllMul::MUL85,        // 4*85 = 340 MHz
        divp: Some(rcc::PllPDiv::DIV2), // 340/2 = 170 MHz
        divq: Some(rcc::PllQDiv::DIV2), // 340/2 = 170 MHz
        divr: Some(rcc::PllRDiv::DIV2), // 340/2 = 170 MHz
    });

    config.rcc = rcc;

    let p = embassy_stm32::init(config);

    /* SPI Bus */

    let mut spi_config = spi::Config::default();
    spi_config.frequency = SPI_FREQUENCY;
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(spi);

    if SPI_BUS.init(spi_bus).is_err() {
        panic!("Failed to init SPI bus!")
    };

    let spi_bus = SPI_BUS.get().await;

    /* ADC stuff */

    let adc = SpiDevice::new(spi_bus, Output::new(p.PA0, Level::High, Speed::VeryHigh));
    let mut adc = Ltc1408_12::new(adc, 5).unwrap();

    let mut adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);
    let _ = adc.read().await; // read unfinished conversions

    /* DRV stuff */

    let drv_cs = Output::new(p.PA11, Level::High, Speed::VeryHigh);
    let drv = SpiDevice::new(spi_bus, drv_cs);
    let mut drv = Drv8323rs::new(drv);
    let mut drv_enable = Output::new(p.PB6, Level::Low, Speed::VeryHigh);
    let n_fault = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);

    drv_enable.set_high();
    Timer::after_millis(2).await;

    assert!(
        check_driver(&mut drv).await.unwrap(),
        "DRV did not function correctly!"
    );

    setup_driver(&mut drv).await.unwrap();

    info!("DRV initialized!");

    // nFAULT handler
    spawner
        .spawn(nfault_handler(n_fault, drv, drv_enable))
        .unwrap();

    /* PWM stuff */

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
        PWM_FREQUENCY,
        CountingMode::CenterAlignedBothInterrupts,
    );

    /* S.BUS */
    let mut sbus_config = usart::Config::default();
    sbus_config.baudrate = 100_000;
    sbus_config.data_bits = DataBits::DataBits8;
    sbus_config.stop_bits = StopBits::STOP2;
    sbus_config.parity = Parity::ParityEven;
    sbus_config.invert_rx = true;
    sbus_config.assume_noise_free = false;

    let sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, sbus_config).unwrap();

    spawner.spawn(radio_receive(sbus)).unwrap();

    /* Control loop (Open loop) */

    // set pwm output
    let frequency: f32 = 110.;

    info!("motor spin at {} Hz, {} rpm", frequency, frequency * 30.,);

    let v_a = helpers::generate_cos(frequency, 0.);
    let v_b = helpers::generate_cos(frequency, -2. * PI / 3.);
    let v_c = helpers::generate_cos(frequency, 2. * PI / 3.);

    let mut angle_filter = DynamicSmootherEcoF32::new(CUTOFF, SAMPLE_RATE, SENSITIVITY);
    // let mut speed_filter = DynamicSmootherEcoF32::new(2.0, SAMPLE_RATE, 0.5);

    let mut last_angle = 0.0;
    let mut last_time = Instant::now();

    loop {
        // enable or disable channels
        let enable = *ENABLE.lock().await;
        if enable {
            pwm.enable(Channel::Ch1);
            pwm.enable(Channel::Ch2);
            pwm.enable(Channel::Ch3);
        } else {
            pwm.disable(Channel::Ch1);
            pwm.disable(Channel::Ch2);
            pwm.disable(Channel::Ch3);
        }

        // trigger ADC conversion
        adc_conv.set_high();
        adc_conv.set_low();

        // read sensor feedback
        let feedback_data = adc.read().await.unwrap();
        let alpha = (feedback_data[0] - 1.24) / 2.0;
        let beta = (feedback_data[1] - 1.24) / 2.0;
        let angle = f32::atan2(alpha, beta) * (180.0 / core::f32::consts::PI);

        // apply filtering
        let angle = angle_filter.tick(angle);

        // calculate rotational frequency
        let dt = last_time.elapsed().as_micros() as f32 * 1e-6;

        let mut diff = angle - last_angle;
        if diff < -180.0 {
            diff += 360.0;
        } else if diff > 180.0 {
            diff -= 360.0;
        }

        let speed = (diff) / (dt * 360.0);
        // let speed = speed_filter.tick(speed);

        info!("angle: {}", angle);
        info!("freq: {}", speed);
        info!("sample_rate: {}", 1e-3 / dt);

        // calculate output
        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_a() + 1.0) / 2.0), Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_b() + 1.0) / 2.0), Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_c() + 1.0) / 2.0), Channel::Ch3);

        // update last values
        last_angle = angle;
        last_time = Instant::now();

        // delay (will be removed)
        // Timer::after_micros(1).await;
    }
}

#[task]
async fn radio_receive(sbus: UartRx<'static, peripherals::USART2, Async>) {
    // create USART DMA buffer
    let mut sbus_buffer = [0; 1024];

    // initialize SBUS
    let sbus = sbus.into_ring_buffered(&mut sbus_buffer);
    let mut sbus = Sbus::new(sbus);

    loop {
        let data = sbus.get_packet().await.unwrap();

        let new_enable = data.ch5() > 1200;

        {
            let mut enable = ENABLE.lock().await;

            if *enable != new_enable {
                if new_enable {
                    info!("enabled!");
                } else {
                    info!("disabled...");
                }
                *enable = new_enable;
            }
        }

        // info!(
        //     "SBUS channels: ch1: {:05}, ch2: {:05}, ch3: {:05}, ch4: {:05}, ch5: {:05}, ch6: {:05}, ch7: {:05}, ch8: {:05}, ch9: {:05}, ch10: {:05}, ch11: {:05}, ch12: {:05}, ch13: {:05}, ch14: {:05}, ch15: {:05}, ch16: {:05}",
        //     data.ch1(),
        //     data.ch2(),
        //     data.ch3(),
        //     data.ch4(),
        //     data.ch5(),
        //     data.ch6(),
        //     data.ch7(),
        //     data.ch8(),
        //     data.ch9(),
        //     data.ch10(),
        //     data.ch11(),
        //     data.ch12(),
        //     data.ch13(),
        //     data.ch14(),
        //     data.ch15(),
        //     data.ch16(),
        // );
    }
}

#[task]
async fn nfault_handler(
    mut n_fault: ExtiInput<'static>,
    mut drv: Drv8323rs<
        SpiDevice<'static, NoopRawMutex, Spi<'static, peripherals::SPI1, Async>, Output<'static>>,
    >,
    mut drv_enable: Output<'static>,
) {
    n_fault.wait_for_low().await;
    report_status(&mut drv).await.unwrap();
    drv_enable.set_low();
    panic!("Shutting down because of DRV error");
}
