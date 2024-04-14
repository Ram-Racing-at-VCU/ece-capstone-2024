#![no_std]
#![no_main]

mod consts;
mod driver;
mod helpers;

use consts::SPI_FREQUENCY;
use driver::{check_driver, report_status, setup_driver};

use drv8323rs::Drv8323rs;
// use ltc1408_12::Ltc1408_12;

use defmt::assert;
// use micromath::F32Ext;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    rcc,
    spi::{self, Spi},
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
async fn main(_spawner: Spawner) {
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

    // nFAULT handler
    // todo: make this into a task (requires a static reference to the SPI bus)
    n_fault.wait_for_low().await;
    report_status(&mut drv).await.unwrap();
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
