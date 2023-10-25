#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _}; // logger & panic handler

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let _p = embassy_stm32::init(Default::default());

    info!("Hello world!");
}
