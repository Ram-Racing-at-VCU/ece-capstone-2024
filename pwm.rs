#![no_std]
#![no_main]

use cortex_m :: Peripherals;
//use panic_rtt_target as _;
use cortex_m_rt :: entry;
use embedded_hal::prelude::*;
use stm_32g4xx_hal as hal;
use hal::stm32;
use {defmt_rtt as _, panic_probe as _};
//use rtt_target :: 


use crate:: hal::{
    prelude::*,
    stm32,
    rcc::Config,
    gpio::{gpioa::PA5, Output, PushPull},
    timer::{Timer, Event, CountDownTimer},
    pwm::{Pwm, C1, C2, C3, C4},
};  


#[entry]

fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);

    let clocks = rcc.cfgr.sysclk(64.mhz()).freeze(&mut flash.acr);

    let mut led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let mut timer = Timer::tim2(dp.TIM2, 1.khz(), clocks, &mut rcc.apb1r1);

    let mut pwm = Pwm::tim1(dp.TIM1, 1.khz(), clocks, &mut rcc.apb2);

    let max = pwm.get_max_duty();

    let mut ch1 = pwm.channel1;
    let mut ch2 = pwm.channel2;
    let mut ch3 = pwm.channel3;
    let mut ch4 = pwm.channel4;

    ch1.set_duty(max / 2);
    ch2.set_duty(max / 2);
    ch3.set_duty(max / 2);
    ch4.set_duty(max / 2);

    ch1.enable();
    ch2.enable();
    ch3.enable();
    ch4.enable();

    loop {
        led.set_high().unwrap();
        timer.wait().unwrap();
        led.set_low().unwrap();
        timer.wait().unwrap();
    }
}

