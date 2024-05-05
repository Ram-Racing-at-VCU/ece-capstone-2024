#![no_std]
#![no_main]

mod consts;
mod driver;
mod helpers;

use core::f32::consts::PI;

use consts::{BANDWIDTH, F_C, F_S, INDUCTANCE, PWM_FREQUENCY, RESISTANCE, SPI_FREQUENCY};
use control_algorithms::{
    foc::{
        clarke_transform, inverse_clarke_transform, inverse_park_transform, park_transform,
        Vector2, Vector3,
    },
    pid::PIDController,
};
use driver::{check_driver, report_status, setup_driver};
use drv8323rs::Drv8323rs;
use ltc1408_12::Ltc1408_12;
use sbus::Sbus;

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};
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

static THROTTLE: Mutex<CriticalSectionRawMutex, Option<f32>> = Mutex::new(None);

#[embassy_executor::main]

// bind USART interrupt
// embassy_stm32::bind_interrupts!(struct Irqs {
//     USART2 => usart::InterruptHandler<peripherals::USART2>;
// });
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

    // let mut speed_filter = MedianFilter::new([0.0; WINDOW_SIZE]);

    let mut i_a_filter = DirectForm2Transposed::<f32>::new(coefficients);

    let mut last_time = Instant::now(); // dt

    // let mut last_angle = 0.0; // differentiating

    let mut pid_cal = PIDController::new(INDUCTANCE * BANDWIDTH, RESISTANCE * BANDWIDTH, 0.0, None);

    info!("Calibrating Initial Angle...");

    pwm.enable(Channel::Ch1);
    pwm.enable(Channel::Ch2);
    pwm.disable(Channel::Ch3);

    let mut angle_cal = 0.0;
    for i in 0..4000 {
        // trigger ADC conversion
        adc_conv.set_high();
        adc_conv.set_low();

        // read sensor feedback
        let feedback_data = adc.read().await.unwrap();
        let new_time = Instant::now();

        let mut alpha = (feedback_data[0] - 1.24) / 2.0;
        let mut beta = (feedback_data[1] - 1.24) / 2.0;

        let mut i_a = (feedback_data[2] - 1.6336) / (10.0 * 2.5e-3);

        // apply filtering
        alpha = alpha_filter.run(alpha);
        beta = beta_filter.run(beta);

        i_a = i_a_filter.run(i_a);

        // calculate angle
        let angle = f32::atan2(alpha, beta) * (180.0 / PI);

        if i > 3000 {
            angle_cal += angle;
        }

        // calculate time delta
        let dt = new_time.duration_since(last_time).as_micros() as f32 * 1e-6;

        let c_t = (pid_cal.output(3.0, i_a, dt) / 12.6).clamp(0.0, 1.0);

        // calculate output
        helpers::set_pwm_duty(&mut pwm, c_t, Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch3);

        // update last values
        last_time = new_time;
    }

    let angle_cal_1 = angle_cal / 1000.0;

    info!("Calibrated A-B: {}", angle_cal_1);

    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch1);
    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch2);
    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch3);

    pwm.enable(Channel::Ch1);
    pwm.disable(Channel::Ch2);
    pwm.enable(Channel::Ch3);

    angle_cal = 0.0;
    for i in 0..4000 {
        // trigger ADC conversion
        adc_conv.set_high();
        adc_conv.set_low();

        // read sensor feedback
        let feedback_data = adc.read().await.unwrap();
        let new_time = Instant::now();

        let mut alpha = (feedback_data[0] - 1.24) / 2.0;
        let mut beta = (feedback_data[1] - 1.24) / 2.0;

        let mut i_a = (feedback_data[2] - 1.6336) / (10.0 * 2.5e-3);

        // apply filtering
        alpha = alpha_filter.run(alpha);
        beta = beta_filter.run(beta);

        i_a = i_a_filter.run(i_a);

        // calculate angle
        let angle = f32::atan2(alpha, beta) * (180.0 / PI);

        if i > 3000 {
            angle_cal += angle;
        }

        // calculate time delta
        let dt = new_time.duration_since(last_time).as_micros() as f32 * 1e-6;

        let c_t = (pid_cal.output(3.0, i_a, dt) / 12.6).clamp(0.0, 1.0);

        // calculate output
        helpers::set_pwm_duty(&mut pwm, c_t, Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch3);

        // update last values
        last_time = new_time;
    }

    let angle_cal_2 = angle_cal / 1000.0;
    info!("Calibrated A-C: {}", angle_cal_2);

    let angle_offset = (angle_cal_1 + angle_cal_2) / 2.0;
    info!("Angle Calibration value: {}", angle_offset);

    info!("Starting control loop!");

    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch1);
    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch2);
    helpers::set_pwm_duty(&mut pwm, 0.0, Channel::Ch3);

    // let mut i_a_filter = DirectForm2Transposed::<f32>::new(coefficients);
    // let mut i_b_filter = DirectForm2Transposed::<f32>::new(coefficients);
    // let mut i_c_filter = DirectForm2Transposed::<f32>::new(coefficients);

    let mut pid_d = PIDController::new(INDUCTANCE * BANDWIDTH, RESISTANCE * BANDWIDTH, 0.0, None);
    let mut pid_q = PIDController::new(INDUCTANCE * BANDWIDTH, RESISTANCE * BANDWIDTH, 0.0, None);

    loop {
        let Some(throttle) = *THROTTLE.lock().await else {
            // info!("disabled...");
            pwm.disable(Channel::Ch1);
            pwm.disable(Channel::Ch2);
            pwm.disable(Channel::Ch3);
            Timer::after_millis(10).await;
            continue;
        };

        pwm.enable(Channel::Ch1);
        pwm.enable(Channel::Ch2);
        pwm.enable(Channel::Ch3);

        // trigger ADC conversion
        adc_conv.set_high();
        adc_conv.set_low();

        // read sensor feedback
        let feedback_data = adc.read().await.unwrap();
        let new_time = Instant::now();

        let mut alpha = (feedback_data[0] - 1.24) / 2.0;
        let mut beta = (feedback_data[1] - 1.24) / 2.0;

        let i_a = (feedback_data[2] - 1.6336) / (10.0 * 2.5e-3);
        let i_b = (feedback_data[3] - 1.6372) / (10.0 * 2.5e-3);
        let i_c = (feedback_data[4] - 1.6395) / (10.0 * 2.5e-3);

        // apply filtering
        alpha = alpha_filter.run(alpha);
        beta = beta_filter.run(beta);

        // i_a = i_a_filter.run(i_a);
        // i_b = i_b_filter.run(i_b);
        // i_c = i_c_filter.run(i_c);

        // calculate angle
        let physical_angle = f32::atan2(alpha, beta) * (180.0 / PI);
        let electrical_angle = (angle_offset - physical_angle) * 2.0;

        // back to radians :(
        let angle_rads = electrical_angle * (PI / 180.0);
        let sin = angle_rads.sin();
        let cos = angle_rads.cos();

        // calculate time delta
        let dt = new_time.duration_since(last_time).as_micros() as f32 * 1e-6;

        // calculate speed
        // let mut _speed = (angle - last_angle) / (dt * 360.0);
        // _speed = speed_filter.run(_speed);

        // info!("angle: {}", angle);
        // info!("speed: {}", speed);
        // info!("sample_rate: {} kHz", 1e-3 / dt);
        // info!("i_a: {}", -i_a * 1e3);
        // info!("i_b: {} mA", i_b * 1e3);
        // info!("i_c: {} mA", i_c * 1e3);

        // Transform currents to stationary frame
        let i_abc = Vector3::<f32>::new(i_a, i_b, i_c);
        let i_xy = clarke_transform(i_abc);
        let i_dq = park_transform(i_xy, sin, cos);

        let i_d = i_dq[0];
        let i_q = i_dq[1];

        // Torque control
        let v_d = pid_d.output(0.0, i_d, dt);
        let v_q = pid_q.output(throttle * 16.95, i_q, dt); // 3A as 100 %

        // Transform back to rotating frame
        let v_dq = Vector2::<f32>::new(v_d, v_q);
        let v_xyz = inverse_park_transform(v_dq, sin, cos);
        let v_abc = inverse_clarke_transform(v_xyz);

        let v_a = (v_abc[0] / 12.6).clamp(-1.0, 1.0);
        let v_b = (v_abc[1] / 12.6).clamp(-1.0, 1.0);
        let v_c = (v_abc[2] / 12.6).clamp(-1.0, 1.0);

        // info!("throttle: {} angle: {}", throttle, electrical_angle);
        // info!("i_a: {} i_b: {} i_c: {}", i_a, i_b, i_c);
        // info!("i_d: {} i_q: {}", i_d, i_q);
        // info!("v_a: {} v_b: {} v_c: {}", v_a, v_b, v_c);
        // info!("v_d: {} v_q: {}", v_d, v_q);

        // calculate output
        helpers::set_pwm_duty(&mut pwm, (v_a + 1.0) / 2.0, Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, (v_b + 1.0) / 2.0, Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, (v_c + 1.0) / 2.0, Channel::Ch3);

        // update last values
        last_time = new_time;
        // last_angle = angle;
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

        let enable = data.ch5() > 1200;
        let throttle = ((data.ch2() as f32) - 1000.0) / 900.0;

        {
            let mut mutex = THROTTLE.lock().await;

            match (mutex.is_some(), enable) {
                (false, false) | (true, true) => (),
                (false, true) => info!("controller enabled!"),
                (true, false) => info!("controller disabled..."),
            };

            if enable {
                *mutex = Some(throttle)
            } else {
                *mutex = None
            };
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

    *THROTTLE.lock().await = None;
    error!("DRV Error!");
    report_status(&mut drv).await.unwrap();
    drv_enable.set_low();
    panic!("Shutting down because of DRV error");
}
