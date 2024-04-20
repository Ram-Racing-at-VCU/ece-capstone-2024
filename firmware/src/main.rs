#![no_std]
#![no_main]

mod consts;
mod driver;
mod helpers;

use core::f32::consts::PI;

use consts::{PWM_FREQUENCY, SPI_FREQUENCY};
use driver::{check_driver, report_status, setup_driver};

use drv8323rs::Drv8323rs;
// use ltc1408_12::Ltc1408_12;

use defmt::{assert, info, println};
// use micromath::F32Ext;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::{task, Spawner};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{DMA1_CH3, TIM1, USART2},
    rcc,
    spi::{self, Spi},
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

// bind USART interrupt
embassy_stm32::bind_interrupts!(
    struct Irqs {
        USART2 => usart::InterruptHandler<USART2>;
    }
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // configure all system clocks to 170 MHz
    let mut rcc = rcc::Config::default();
    rcc.mux = rcc::ClockSrc::PLL;
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

    /* SPI Bus */

    let mut spi_config = spi::Config::default();
    spi_config.frequency = SPI_FREQUENCY;
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(spi);

    /* S.BUS */
    let mut sbus_config = usart::Config::default();
    sbus_config.baudrate = 100_000;
    sbus_config.data_bits = DataBits::DataBits8;
    sbus_config.stop_bits = StopBits::STOP2;
    sbus_config.parity = Parity::ParityEven;
    sbus_config.invert_rx = true;

    let sbus = UartRx::new(p.USART2, Irqs, p.PA3, p.DMA1_CH3, sbus_config).unwrap();

    /* DRV stuff */

    let drv_cs = Output::new(p.PA11, Level::High, Speed::VeryHigh);
    let drv = SpiDevice::new(&spi_bus, drv_cs);
    let mut drv = Drv8323rs::new(drv);
    let mut drv_enable = Output::new(p.PB6, Level::Low, Speed::VeryHigh);
    let n_fault = Input::new(p.PB0, Pull::Up);
    let mut n_fault = ExtiInput::new(n_fault, p.EXTI0);

    drv_enable.set_high();
    Timer::after_millis(2).await;

    assert!(
        check_driver(&mut drv).await.unwrap(),
        "DRV did not function correctly!"
    );

    setup_driver(&mut drv).await.unwrap();

    info!("DRV initialized!");

    /* PWM stuff */

    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let ch2n = ComplementaryPwmPin::new_ch2(p.PA12, OutputType::PushPull);
    let ch3 = PwmPin::new_ch3(p.PA10, OutputType::PushPull);
    let ch3n = ComplementaryPwmPin::new_ch3(p.PF0, OutputType::PushPull);

    let pwm = ComplementaryPwm::new(
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

    spawner.spawn(spwm(pwm, sbus)).unwrap();

    // nFAULT handler
    // todo: make this into a task (requires a static reference to the SPI bus)
    n_fault.wait_for_low().await;
    report_status(&mut drv).await.unwrap();
    drv_enable.set_low();
    panic!("Shutting down because of DRV error");

    /* ADC stuff */

    // todo: make this share the bus with the DRV somehow

    // let mut adc = Ltc1408_12::new(spi, 5).unwrap();

    // let mut adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);

    // loop {
    //     // trigger conversion
    //     adc_conv.set_high();
    //     adc_conv.set_low();

    //     // read data
    //     let adc_data = adc.read().await.unwrap();

    //     // assign variables & apply scaling
    //     let alpha = (adc_data[0] - 1.24) / 2.0;
    //     let beta = (adc_data[1] - 1.24) / 2.0;
    //     let angle = f32::atan2(alpha, beta) * (180.0 / core::f32::consts::PI);

    //     // print output
    //     info!("Got alpha: {}, beta: {}, angle: {}", alpha, beta, angle);

    //     Timer::after_millis(50).await;
    // }
}

#[task]
async fn spwm(mut pwm: ComplementaryPwm<'static, TIM1>, _sbus: UartRx<'static, USART2, DMA1_CH3>) {
    // create USART DMA buffer
    // let mut sbus_buffer = [0; 64];

    // initialize SBUS
    // let mut sbus = sbus.into_ring_buffered(&mut sbus_buffer);
    // let mut sbus = Sbus::new(sbus);

    // enable channels
    pwm.enable(Channel::Ch1);
    pwm.enable(Channel::Ch2);
    pwm.enable(Channel::Ch3);

    // set pwm output
    let frequency: f32 = 110.;

    println!("motor spin at {} Hz, {} rpm", frequency, frequency * 30.,);

    let v_a = helpers::generate_cos(frequency, 0.);
    let v_b = helpers::generate_cos(frequency, -2. * PI / 3.);
    let v_c = helpers::generate_cos(frequency, 2. * PI / 3.);

    loop {
        // let mut bytes = [0; 25];
        // sbus.read(&mut bytes).await.unwrap();
        // info!("bytes: {}", bytes);

        // info!("waiting for data :(");
        // let data = sbus.get_packet().await.unwrap();

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

        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_a() + 1.0) / 2.0), Channel::Ch1);
        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_b() + 1.0) / 2.0), Channel::Ch2);
        helpers::set_pwm_duty(&mut pwm, 0.2 * ((v_c() + 1.0) / 2.0), Channel::Ch3);
        Timer::after_micros(1).await;
    }
}
