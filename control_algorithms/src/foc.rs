//! Field-oriented control functions

use micromath::F32Ext;
use nalgebra as na;

// /// Apply clarke transform to input signal
// pub fn clarke_transform(input_signal: na::SVector<f32, 3>) -> na::SVector<f32, 2> {
//     2f32 / 3f32
//         * na::SMatrix::<f32, 2, 3>::new(
//             // Defining Park matrix
//             1f32,
//             -0.5f32,
//             -0.5f32,
//             0f32,
//             3f32.sqrt() / 2f32,
//             -(3f32.sqrt()) / 2f32,
//         )
//         * input_signal
// }

// /// Apply inverse clarke transform to input signal
// pub fn inverse_clarke_transform(input_signal: na::SVector<f32, 2>) -> na::SVector<f32, 3> {
//     na::SMatrix::<f32, 3, 2>::new(
//         1f32,
//         0f32,
//         -0.5f32,
//         3f32.sqrt() / 2f32,
//         -0.5f32,
//         -(3f32.sqrt()) / 2f32,
//     ) * input_signal
// }

// /// Apply park transform to input signal
// pub fn park_transform(input_signal: na::SVector<f32, 2>, phase_angle: f32) -> na::SVector<f32, 2> {
//     na::SMatrix::<f32, 2, 2>::new(
//         // Defining clarke matrix
//         phase_angle.cos(),
//         phase_angle.sin(),
//         -phase_angle.sin(),
//         phase_angle.cos(),
//     ) * input_signal
// }

/// Apply clarke and park transform to input signal
pub fn dq_transform(v_a: f32, v_b: f32, v_c: f32, angle: f32) -> (f32, f32) {
    let s: f32 = angle.sin();
    let c: f32 = angle.cos();

    let v_d: f32 = (2. / 3.)
        * (v_a * c + ((3f32.sqrt() / 2.) * s - (1. / 2.) * c) * v_b
            - ((1. / 2.) * c + (3f32.sqrt() / 2.) * s) * v_c);

    let v_q: f32 = (2. / 3.)
        * (-v_a * s
            + ((3f32.sqrt() / 2.) * c + (1. / 2.) * s) * v_b
            + ((1. / 2.) * s - (3f32.sqrt() / 2.) * c) * v_c);

    (v_d, v_q)
}

/// Apply inverse park transform to input signal
pub fn inverse_park_transform(v_d: f32, v_q: f32, angle: f32) -> (f32, f32) {
    let v_alpha: f32 = v_d * angle.cos() - v_q * angle.sin();
    let v_beta: f32 = v_d * angle.sin() + v_q * angle.cos();
    (v_alpha, v_beta)
}
