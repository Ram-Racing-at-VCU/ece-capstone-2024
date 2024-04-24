//! Digital filter implementations

use core::ops::{Index, IndexMut};

/// Ring buffer
pub struct RingBuffer<const SIZE: usize, T> {
    /// Data
    data: [T; SIZE],
    /// Index
    idx: usize,
}

impl<const SIZE: usize, T> RingBuffer<SIZE, T> {
    /// Create the buffer
    pub fn new(data: [T; SIZE]) -> Self {
        Self { data, idx: 0 }
    }

    /// Insert a new element
    pub fn insert(&mut self, new: T) {
        self.data[self.idx] = new;
        self.idx = (self.idx + 1) % SIZE;
    }
}

impl<const SIZE: usize, T: Copy> RingBuffer<SIZE, T> {
    /// Copy the data into an array
    pub fn copy(&self) -> [T; SIZE] {
        self.data
    }
}

impl<const SIZE: usize, T> Index<usize> for RingBuffer<SIZE, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(index + self.idx) % SIZE]
    }
}

impl<const SIZE: usize, T> IndexMut<usize> for RingBuffer<SIZE, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[(index + self.idx) % SIZE]
    }
}

/// Simple averaging filter with a sliding window backed by a ring buffer
pub struct AverageFilter<const SIZE: usize> {
    /// Data
    data: RingBuffer<SIZE, f32>,
}

impl<const SIZE: usize> AverageFilter<SIZE> {
    /// Initialize the filter, starting with 0
    pub fn new(initial_values: [f32; SIZE]) -> Self {
        Self {
            data: RingBuffer::new(initial_values),
        }
    }

    /// Get the next filtered output
    pub fn run(&mut self, new_value: f32) -> f32 {
        self.data.insert(new_value);

        let mut total = 0.0;
        for i in 0..SIZE {
            total += self.data[i];
        }

        total / SIZE as f32
    }
}

/// Median filter with a sliding window backed by a ring buffer and sorted using insertion sort
pub struct MedianFilter<const SIZE: usize> {
    /// Data
    data: RingBuffer<SIZE, f32>,
}

impl<const SIZE: usize> MedianFilter<SIZE> {
    /// Initialize the filter
    pub fn new(initial_values: [f32; SIZE]) -> Self {
        Self {
            data: RingBuffer::new(initial_values),
        }
    }

    /// Get the next filtered output
    pub fn run(&mut self, new_value: f32) -> f32 {
        self.data.insert(new_value);

        let mut data = self.data.copy();
        insertion_sort(&mut data);

        if SIZE % 2 == 0 {
            // e.g. 4: 2 & 3 -> 1 & 2
            (data[(SIZE / 2) - 1] + data[(SIZE) / 2]) / 2.0
        } else {
            // e.g. 5: 3 -> 2
            data[SIZE / 2]
        }
    }
}

/// Insertion sort algorithm from Wikipedia
fn insertion_sort<const SIZE: usize, T: PartialOrd + Copy>(array: &mut [T; SIZE]) {
    for i in 0..SIZE {
        let x = array[i];
        let mut j = i;

        while j > 0 && array[j - 1] > x {
            array[j] = array[j - 1];
            j -= 1;
        }

        array[j] = x;
    }
}
