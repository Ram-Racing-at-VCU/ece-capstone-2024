#![allow(dead_code)]

use core::f32::consts::PI;

use drv8323rs::{registers as driver_registers, Drv8323rs, EditRegister};
use sbus::Sbus;

use defmt::debug;
use embassy_stm32::{
    timer::{
        complementary_pwm::ComplementaryPwm, Channel, ComplementaryCaptureCompare16bitInstance,
    },
    usart::{self, UartRx},
};
use embassy_time::Instant;
use micromath::F32Ext;

pub fn generate_angle(frequency: f32) -> impl Fn() -> f32 {
    move || {
        let t = Instant::now().as_micros() as f32 / 1e6;
        (2. * PI * frequency * t) % (2. * PI)
    }
}

/// Frequency: [Hz], Phase: [Rad]
pub fn generate_cos(frequency: f32, phase: f32) -> impl Fn() -> f32 {
    let angle = generate_angle(frequency);

    move || (angle() + phase).cos()
}

pub fn map_range(val: f32, before: (f32, f32), after: (f32, f32)) -> f32 {
    (val - before.0) * (after.1 - after.0) / (before.1 - before.0) + after.0
}

pub fn set_pwm_duty<T>(pwm: &mut ComplementaryPwm<T>, frac: f32, channel: Channel)
where
    T: ComplementaryCaptureCompare16bitInstance,
{
    let max = pwm.get_max_duty() as f32;
    let duty = (max * frac) as u16;
    pwm.set_duty(channel, duty);
}

pub async fn driver_setup<T: embedded_hal_async::spi::SpiDevice<u8>>(driver: &mut Drv8323rs<T>) {
    // note: this is placeholder code just to test the SPI bus
    // replace with actual setup code later...

    driver
        .edit(|r: &mut driver_registers::Control| {
            r.set_pwm_mode(driver_registers::PwmMode::_3x);
            r.set_brake(false);
            debug!("control: {}", r.into_bytes());
        })
        .await
        .unwrap();
}

pub struct UartRxWrapper<'d, T: usart::BasicInstance, RxDma: usart::RxDma<T>> {
    inner: UartRx<'d, T, RxDma>,
}

impl<'d, T: usart::BasicInstance, RxDma: usart::RxDma<T>> UartRxWrapper<'d, T, RxDma> {
    pub fn new(sbus: UartRx<'d, T, RxDma>) -> Self {
        Self { inner: sbus }
    }
}

impl<'d, T: usart::BasicInstance, RxDma: usart::RxDma<T>> embedded_io_async::ErrorType
    for UartRxWrapper<'d, T, RxDma>
{
    type Error = usart::Error;
}

impl<'d, T: usart::BasicInstance, RxDma: usart::RxDma<T>> embedded_io_async::Read
    for UartRxWrapper<'d, T, RxDma>
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.read_until_idle(buf).await
    }
}

pub fn usart_to_sbus<T: usart::BasicInstance, RxDma: usart::RxDma<T>>(
    usart: UartRx<'_, T, RxDma>,
) -> Sbus<UartRxWrapper<'_, T, RxDma>> {
    Sbus::new(UartRxWrapper::new(usart))
}
