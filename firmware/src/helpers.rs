#![allow(dead_code)]

use core::f32::consts::PI;

use embassy_stm32::timer::{
    complementary_pwm::ComplementaryPwm, AdvancedInstance4Channel, Channel,
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
    T: AdvancedInstance4Channel,
{
    let max = pwm.get_max_duty() as f32;
    let duty = (max * frac) as u16;
    pwm.set_duty(channel, duty);
}
