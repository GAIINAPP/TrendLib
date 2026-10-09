use crate::core::error::TlError;

/// A live indicator value. Streams are plain values: no global state, no
/// shared caches, and `clone` is a fully independent fork (D9).
pub trait Stream: Clone + Sized {
    type Params;
    /// History as it actually arrives: one contiguous column per input, which
    /// is what NumPy hands over. Nothing is transposed to open a stream.
    type History<'a>;
    /// One bar: `[f64; inputs]`, in declaration order.
    type Bar: Copy;
    /// One row of outputs: `[f64; outputs]`, in declaration order.
    type Value: Copy;

    /// Warm up from history. Needs `lookback + 1` valid bars after any leading
    /// warm-up rows, otherwise `TlError::InsufficientHistory`.
    fn open(history: Self::History<'_>, params: &Self::Params) -> Result<Self, TlError>;

    /// Commit one closed bar and return its value.
    fn update(&mut self, bar: Self::Bar) -> Result<Self::Value, TlError>;

    /// What `update(bar)` would return. Changes nothing.
    fn peek(&self, bar: Self::Bar) -> Result<Self::Value, TlError>;

    /// The last committed value, or `None` before the first one.
    fn value(&self) -> Option<Self::Value>;

    /// Bars consumed so far, history included.
    fn bars_seen(&self) -> u64;
}
