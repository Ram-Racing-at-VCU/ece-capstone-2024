// Change `extern crate` to use when using crates
extern crate micromath as um; // no-std library for trig operations
extern crate nalgebra as na; // no-std library for matrix arithmatic

#[derive(Default)] // Default Constructor for PID Class; initialize all fields to 0.
pub struct PIDController {
    pub k_p: f32,        // Proportional constant
    pub k_i: f32,        // Integral constant
    pub k_d: f32,        // Derivative constant
    pub dt: f32,         // Sampling time
    error: f32,          // Current error value (internal to object)
    previous_error: f32, // Previous error value (internal to object)
    output: f32,         // Output value (internal to object)
}

impl PIDController {
    // Constructor with field values
    pub fn new(k_p: f32, k_i: f32, k_d: f32, dt: f32) -> Self {
        Self {
            k_p,
            k_i,
            k_d,
            dt,
            ..Default::default() // Initialize other fields to zero, as user should not have access to those fields
        }
    }

    // Modify controller parameters
    pub fn modify(&mut self, k_p: f32, k_i: f32, k_d: f32, dt: f32) {
        self.k_p = k_p;
        self.k_i = k_i;
        self.k_d = k_d;
        self.dt = dt;
    }

    // Update internal states of controller, and return control signal
    pub fn update(&mut self, input: f32, measurement: f32) -> f32 {
        self.error = input - measurement; // Calculate current value of error signal

        self.output = self.k_p * self.error                                 // Proportional component (constant * error)
            + self.k_i * (self.error + self.previous_error) * self.dt       // Integral component (constant * integral(error, dt))
            + self.k_d * (self.error - self.previous_error) / self.dt; // Derivative component (constant * d/dt(error))

        self.previous_error = self.error; // Update previous error value to use in next iteration

        return self.output;
    }
}

// Apply clarke transform to input signal
fn clarke_transform(input_signal: na::SVector<f32, 3>) -> na::SVector<f32, 2> {
    return 2f32 / 3f32
        * na::SMatrix::<f32, 2, 3>::new(
            // Defining Park matrix
            1f32,
            -0.5f32,
            -0.5f32,
            0f32,
            3f32.sqrt() / 2f32,
            3f32.sqrt() / 2f32,
        )
        * input_signal;
}

// Apply inverse clarke transform to input signal
fn inverse_clarke_transform(input_signal: na::SVector<f32, 2>) -> na::SVector<f32, 3> {
    return na::SMatrix::<f32, 3, 2>::new(
        1f32,
        0f32,
        -0.5f32,
        3f32.sqrt() / 2f32,
        -0.5f32,
        -3f32.sqrt() / 2f32,
    ) * input_signal;
}

// Apply park transform to input signal
fn park_transform(input_signal: na::SVector<f32, 2>, phase_angle: f32) -> na::SVector<f32, 2> {
    return na::SMatrix::<f32, 2, 2>::new(
        // Defining clarke matrix
        phase_angle.cos(),
        phase_angle.sin(),
        -phase_angle.sin(),
        phase_angle.cos(),
    ) * input_signal;
}

// Apply inverse park transfrom to input signal
fn inverse_park_transform(
    input_signal: na::SVector<f32, 2>,
    phase_angle: f32,
) -> na::SVector<f32, 2> {
    return na::SMatrix::<f32, 2, 2>::new(
        // Defining clarke matrix
        phase_angle.cos(),
        -phase_angle.sin(),
        phase_angle.sin(),
        phase_angle.cos(),
    ) * input_signal;
}
