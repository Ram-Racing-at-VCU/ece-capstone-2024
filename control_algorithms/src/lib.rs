//! Implementations of control algorithms and functions relating to them.

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use micromath::F32Ext;
use nalgebra as na; // no-std library for matrix arithmetic

/// A simple implementation of the standard PID controller
#[derive(Default)]
pub struct PIDController {
    /// Proportional constant
    pub k_p: f32,
    /// Integral constant     
    pub k_i: f32,
    /// Derivative constant
    pub k_d: f32,
    /// Sampling time
    pub dt: f32,
    /// Current error value
    error: f32,
    /// Previous error value
    previous_error: f32,
    /// Output value
    output: f32,
}

impl PIDController {
    /// Constructor with field values
    pub fn new(k_p: f32, k_i: f32, k_d: f32, dt: f32) -> Self {
        Self {
            k_p,
            k_i,
            k_d,
            dt,
            ..Default::default() // Initialize other fields to zero, as user should not have access to those fields
        }
    }

    /// Modify controller parameters
    pub fn modify(&mut self, k_p: f32, k_i: f32, k_d: f32, dt: f32) {
        self.k_p = k_p;
        self.k_i = k_i;
        self.k_d = k_d;
        self.dt = dt;
    }

    /// Update internal states of controller, and return control signal
    pub fn update(&mut self, input: f32, measurement: f32) -> f32 {
        self.error = input - measurement; // Calculate current value of error signal

        self.output = self.k_p * self.error                                 // Proportional component (constant * error)
            + self.k_i * (self.error + self.previous_error) * self.dt       // Integral component (constant * integral(error, dt))
            + self.k_d * (self.error - self.previous_error) / self.dt; // Derivative component (constant * d/dt(error))

        self.previous_error = self.error; // Update previous error value to use in next iteration

        self.output
    }
}

/// Apply clarke transform to input signal
pub fn clarke_transform(input_signal: na::SVector<f32, 3>) -> na::SVector<f32, 2> {
    2f32 / 3f32
        * na::SMatrix::<f32, 2, 3>::new(
            // Defining Park matrix
            1f32,
            -0.5f32,
            -0.5f32,
            0f32,
            3f32.sqrt() / 2f32,
            3f32.sqrt() / 2f32,
        )
        * input_signal
}

/// Apply inverse clarke transform to input signal
pub fn inverse_clarke_transform(input_signal: na::SVector<f32, 2>) -> na::SVector<f32, 3> {
    na::SMatrix::<f32, 3, 2>::new(
        1f32,
        0f32,
        -0.5f32,
        3f32.sqrt() / 2f32,
        -0.5f32,
        -(3f32.sqrt()) / 2f32,
    ) * input_signal
}

/// Apply park transform to input signal
pub fn park_transform(input_signal: na::SVector<f32, 2>, phase_angle: f32) -> na::SVector<f32, 2> {
    na::SMatrix::<f32, 2, 2>::new(
        // Defining clarke matrix
        phase_angle.cos(),
        phase_angle.sin(),
        -phase_angle.sin(),
        phase_angle.cos(),
    ) * input_signal
}

/// Apply inverse park transform to input signal
pub fn inverse_park_transform(
    input_signal: na::SVector<f32, 2>,
    phase_angle: f32,
) -> na::SVector<f32, 2> {
    na::SMatrix::<f32, 2, 2>::new(
        // Defining clarke matrix
        phase_angle.cos(),
        -phase_angle.sin(),
        phase_angle.sin(),
        phase_angle.cos(),
    ) * input_signal
}
