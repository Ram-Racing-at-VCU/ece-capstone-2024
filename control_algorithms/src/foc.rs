//! Field-oriented control functions

use micromath::F32Ext;
use nalgebra as na;

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
            -(3f32.sqrt()) / 2f32,
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
