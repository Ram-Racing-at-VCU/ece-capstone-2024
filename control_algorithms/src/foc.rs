//! Field-oriented control functions
use micromath::F32Ext;
pub use nalgebra::{Matrix2, Matrix2x3, Matrix3x2, Vector2, Vector3};

/// Apply clarke transform to input signal
pub fn clarke_transform(input_signal: Vector3<f32>) -> Vector2<f32> {
    2f32 / 3f32
        * Matrix2x3::<f32>::new(
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
pub fn inverse_clarke_transform(input_signal: Vector2<f32>) -> Vector3<f32> {
    Matrix3x2::<f32>::new(
        1f32,
        0f32,
        -0.5f32,
        3f32.sqrt() / 2f32,
        -0.5f32,
        -(3f32.sqrt()) / 2f32,
    ) * input_signal
}

/// Apply park transform to input signal
pub fn park_transform(input_signal: Vector2<f32>, angle_sin: f32, angle_cos: f32) -> Vector2<f32> {
    Matrix2::<f32>::new(
        // Defining clarke matrix
        angle_cos, angle_sin, -angle_sin, angle_cos,
    ) * input_signal
}

/// Apply inverse park transform to input signal
pub fn inverse_park_transform(
    input_signal: Vector2<f32>,
    angle_sin: f32,
    angle_cos: f32,
) -> Vector2<f32> {
    Matrix2::<f32>::new(angle_cos, -angle_sin, angle_sin, angle_cos) * input_signal
}

// /// Apply clarke and park transform to input signal
// pub fn dq_transform(v_a: f32, v_b: f32, v_c: f32, angle: f32) -> (f32, f32) {
//     let s: f32 = angle.sin();
//     let c: f32 = angle.cos();

//     let v_d: f32 = (2. / 3.)
//         * (v_a * c + ((3f32.sqrt() / 2.) * s - (1. / 2.) * c) * v_b
//             - ((1. / 2.) * c + (3f32.sqrt() / 2.) * s) * v_c);

//     let v_q: f32 = (2. / 3.)
//         * (-v_a * s
//             + ((3f32.sqrt() / 2.) * c + (1. / 2.) * s) * v_b
//             + ((1. / 2.) * s - (3f32.sqrt() / 2.) * c) * v_c);

//     (v_d, v_q)
// }

// /// Apply inverse park transform to input signal
// pub fn inverse_park_transform(v_d: f32, v_q: f32, angle: f32) -> (f32, f32) {
//     let v_alpha: f32 = v_d * angle.cos() - v_q * angle.sin();
//     let v_beta: f32 = v_d * angle.sin() + v_q * angle.cos();
//     (v_alpha, v_beta)
// }
