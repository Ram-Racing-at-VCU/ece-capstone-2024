//! Digital filter implementations

/// Simple averaging filter with a window
pub struct AverageFilter<const SIZE: usize> {
    /// Data
    data: [f32; SIZE],
}

impl<const SIZE: usize> AverageFilter<SIZE> {
    /// Initialize the filter, starting with 0
    pub fn new(initial_values: [f32; SIZE]) -> Self {
        Self {
            data: initial_values,
        }
    }

    /// Get the next output
    pub fn update(&mut self, new_value: f32) -> f32 {
        let mut total = 0.0;

        for i in 0..SIZE {
            if i < SIZE - 1 {
                self.data[i] = self.data[i + 1]
            } else {
                self.data[i] = new_value
            }
            total += self.data[i];
        }

        total / SIZE as f32
    }
}
