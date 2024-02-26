//! Proportional-integral-derivative based control algorithm

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
    /// Previous error value
    previous_error: f32,
    /// Accumulated error
    accumulated_error: f32,
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
        // Calculate current error
        let error = input - measurement;

        // Update integral term
        self.accumulated_error += error * self.dt;

        // Proportional component (constant * error)
        let p = self.k_p * error;
        // Integral component (constant * integral(error, dt))
        let i = self.k_i * self.accumulated_error * self.dt;
        // Derivative component (constant * d/dt(error))
        let d = self.k_d * (error - self.previous_error) / self.dt;
        // Update previous error value to be used in next iteration
        self.previous_error = error;

        // Return controller output
        p + i + d
    }
}
