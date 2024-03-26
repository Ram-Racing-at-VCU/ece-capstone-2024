#![no_std]
#![no_main]

mod helpers;

use ltc1408_12::Ltc1408_12;

use defmt::*;

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    rcc,
    spi::{self, Spi},
    time, Config,
};

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

    info!("Initialized system!");

    let mut spi_config = spi::Config::default();
    spi_config.frequency = time::mhz(10);
    spi_config.mode = spi::MODE_1;

    let spi = Spi::new(
        p.SPI1, p.PB3, p.PB5, p.PB4, p.DMA1_CH1, p.DMA1_CH2, spi_config,
    );

    let mut adc = Ltc1408_12::new(spi, 5).unwrap();

    let mut adc_conv = Output::new(p.PA2, Level::Low, Speed::Low);

    loop {
        adc_conv.set_high();
        adc_conv.set_low();

        let data = adc.read().await.unwrap()[0];

        let data = (data - 1.24) / 2.0;

        info!("Got ADC data: {:?}", data);
    }
}
