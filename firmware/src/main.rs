#![no_std]
#![no_main]

mod consts;
mod driver;
mod helpers;

use core::f32::consts::PI;

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};
use consts::{F_C, F_S, PWM_FREQUENCY, SPI_FREQUENCY, WINDOW_SIZE};
use control_algorithms::{filters::MedianFilter, pid::PIDController};
use driver::{check_driver, report_status, setup_driver};

use drv8323rs::Drv8323rs;
use ltc1408_12::Ltc1408_12;
use sbus::Sbus;

use defmt::{assert, error, info};
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
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
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

static SPI_BUS: OnceLock<Mutex<CriticalSectionRawMutex, Spi<'static, peripherals::SPI1, Async>>> =
    OnceLock::new();

static ENABLE: Mutex<CriticalSectionRawMutex, bool> = Mutex::new(false);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    /* Clock configuration (170 MHz sysclk) */
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

    /* SPI Bus Setup */

    let mut spi_config = spi::Config::default();
    spi_config.frequency = SPI_FREQUENCY;
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi_bus: Mutex<CriticalSectionRawMutex, _> = Mutex::new(spi);

    // send to static location for resource sharing
    if SPI_BUS.init(spi_bus).is_err() {
        panic!("Failed to init SPI bus!")
    };

    let spi_bus = SPI_BUS.get().await;

    /* ADC Setup */

    let adc = SpiDevice::new(spi_bus, Output::new(p.PA0, Level::High, Speed::VeryHigh));
    let mut adc = Ltc1408_12::new(adc, 5).unwrap();

    let mut adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);
    let _ = adc.read().await; // flush partially completed conversions

    /* DRV Setup */

    let drv_cs = Output::new(p.PA11, Level::High, Speed::VeryHigh);
    let drv = SpiDevice::new(spi_bus, drv_cs);
    let mut drv = Drv8323rs::new(drv);
    let mut drv_enable = Output::new(p.PB6, Level::Low, Speed::VeryHigh);
    let n_fault = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);
    let mut drv_cal = Output::new(p.PB7, Level::Low, Speed::VeryHigh);

    drv_enable.set_high();
    Timer::after_millis(2).await;

    assert!(
        check_driver(&mut drv).await.unwrap(),
        "DRV did not function correctly!"
    );

    drv_cal.set_high();
    Timer::after_micros(100).await;
    drv_cal.set_low();

    setup_driver(&mut drv).await.unwrap();

    info!("DRV initialized!");

    // spawn fault handler
    spawner
        .spawn(nfault_handler(n_fault, drv, drv_enable))
        .unwrap();

    /* PWM Setup */

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

    /* SBUS Setup (remote controller) */

    let mut sbus_config = usart::Config::default();
    sbus_config.baudrate = 100_000;
    sbus_config.data_bits = DataBits::DataBits8;
    sbus_config.stop_bits = StopBits::STOP2;
    sbus_config.parity = Parity::ParityEven;
    sbus_config.invert_rx = true;
    sbus_config.assume_noise_free = false;

    let sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, sbus_config).unwrap();

    // spawn receive task
    spawner.spawn(radio_receive(sbus)).unwrap();

    /* Control loop (Open loop) */

    // set pwm output
    let frequency: f32 = 110.;

    info!(
        "motor target speed: {} Hz, {} rpm",
        frequency,
        frequency * 30.,
    );

    let coefficients =
        Coefficients::<f32>::from_params(Type::LowPass, F_S.khz(), F_C.hz(), Q_BUTTERWORTH_F32)
            .unwrap();

    let mut alpha_filter = DirectForm2Transposed::<f32>::new(coefficients);
    let mut beta_filter = DirectForm2Transposed::<f32>::new(coefficients);

    let mut speed_filter = MedianFilter::new([0.0; WINDOW_SIZE]);

    let mut i_a_filter = DirectForm2Transposed::<f32>::new(coefficients);
    let mut i_b_filter = DirectForm2Transposed::<f32>::new(coefficients);
    let mut i_c_filter = DirectForm2Transposed::<f32>::new(coefficients);

    let mut last_time = Instant::now(); // dt

    let mut last_angle = 0.0; // differentiating

    let mut _qn = 0.0;
    let mut _offset = 0.0;

    let bw = 1e3;
    let l = 2.56e-6;
    let r = 6.2832e-3;
    let mut pid = PIDController::new(l * bw, r * bw, 0.0, None);

    info!("Starting control loop!");

    loop {
        // enable or disable channels
        let enable = *ENABLE.lock().await;
        if enable {
            pwm.enable(Channel::Ch1);
            pwm.enable(Channel::Ch2);
            pwm.enable(Channel::Ch3);
        } else {
            info!("disabled...");
            pwm.disable(Channel::Ch1);
            pwm.disable(Channel::Ch2);
            pwm.disable(Channel::Ch3);
            Timer::after_millis(10).await;
            continue;
        }

        // trigger ADC conversion
        adc_conv.set_high();
        adc_conv.set_low();

        // read sensor feedback
        let feedback_data = adc.read().await.unwrap();
        let new_time = Instant::now();

        let mut alpha = (feedback_data[0] - 1.24) / 2.0;
        let mut beta = (feedback_data[1] - 1.24) / 2.0;

        let mut i_a = (1.6336 - feedback_data[2]) / (10.0 * 2.5e-3);
        let mut i_b = (1.6372 - feedback_data[3]) / (10.0 * 2.5e-3);
        let mut i_c = (1.6395 - feedback_data[4]) / (10.0 * 2.5e-3);

        // apply filtering
        alpha = alpha_filter.run(alpha);
        beta = beta_filter.run(beta);

        i_a = i_a_filter.run(i_a);
        i_b = i_b_filter.run(i_b);
        i_c = i_c_filter.run(i_c);

        // calculate angle
        let angle = f32::atan2(alpha, beta) * (180.0 / PI);

        // calculate time delta
        let dt = new_time.duration_since(last_time).as_micros() as f32 * 1e-6;

        // calculate speed
        let mut _speed = (angle - last_angle) / (dt * 360.0);
        _speed = speed_filter.run(_speed);

        // info!("{}, {}, {}, {},", i_a, i_b, i_c, dt);

        _qn += (f32::abs(i_a) + f32::abs(i_b) + f32::abs(i_c)) / 3.0;
        _offset += f32::abs(i_a + i_b + i_c);

        let c_t = (pid.output(3.0, -i_a, dt) / 12.6).clamp(0.0, 1.0);

        // info!("angle: {}", angle);
        // info!("speed: {}", speed);
        // info!("sample_rate: {} kHz", 1e-3 / dt);
        info!("i_a: {}", -i_a * 1e3);
        // info!("i_b: {} mA", i_b * 1e3);
        // info!("i_c: {} mA", i_c * 1e3);
        info!("c_t: {}", c_t);

        // info!("quantized noise: {} A", qn / 1000.0);
        // info!("offset: {} A", offset / 1000.0);
        _qn = 0.0;
        _offset = 0.0;

        // calculate output
        helpers::set_pwm_duty(&mut pwm, c_t, Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch3);

        // update last values
        last_time = new_time;
        last_angle = angle;
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
        SpiDevice<
            'static,
            CriticalSectionRawMutex,
            Spi<'static, peripherals::SPI1, Async>,
            Output<'static>,
        >,
    >,
    mut drv_enable: Output<'static>,
) {
    n_fault.wait_for_low().await;
    *ENABLE.lock().await = false;
    error!("DRV Error!");
    report_status(&mut drv).await.unwrap();
    drv_enable.set_low();
    panic!("Shutting down because of DRV error");
}
