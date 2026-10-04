/// A float output buffer aligned to the input, pre-filled with warm-up rows.
pub fn nan_filled(len: usize) -> Vec<f64> {
    vec![f64::NAN; len]
}

/// An integer output buffer aligned to the input, pre-filled with warm-up rows.
pub fn zero_filled(len: usize) -> Vec<i32> {
    vec![0; len]
}
