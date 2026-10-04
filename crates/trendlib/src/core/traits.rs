use crate::core::error::TlError;

/// One indicator: parameters, warm-up length, batch computation and a stream.
pub trait Indicator {
    /// The function name, identical to the folder name and to `spec.yaml`.
    const NAME: &'static str;

    type Params: Default + Clone;
    /// `&[f64]` for a single series, an `Ohlcv` view for bar data.
    type Input<'a>;
    /// `Vec<f64>`, or a struct of vectors for a multi-output indicator.
    type Output;
    type Stream: Stream<Params = Self::Params>;

    /// Reject parameters outside the ranges in `spec.yaml`.
    fn validate(params: &Self::Params) -> Result<(), TlError>;

    /// Warm-up rows before the first defined value, counted from the first
    /// valid bar (`docs/CONVENTIONS.md` § 2).
    fn lookback(params: &Self::Params) -> usize;

    fn batch(input: Self::Input<'_>, params: &Self::Params) -> Result<Self::Output, TlError>;

    /// Batch output and a stream positioned at its last bar, in one pass.
    fn open_and_fill(
        input: Self::Input<'_>,
        params: &Self::Params,
    ) -> Result<(Self::Stream, Self::Output), TlError>;
}

/// A live indicator value. Streams are plain values: no global state, no
/// shared caches, and `clone` is a fully independent fork (D9).
pub trait Stream: Clone + Sized {
    type Params;
    /// `f64` for a single series, a small `Copy` struct for bar data.
    type Bar: Copy;
    type Value: Copy;

    /// Warm up from history. Needs `lookback + 1` valid bars after any leading
    /// warm-up rows, otherwise `TlError::InsufficientHistory`.
    fn open(history: &[Self::Bar], params: &Self::Params) -> Result<Self, TlError>;

    /// Commit one closed bar and return its value.
    fn update(&mut self, bar: Self::Bar) -> Result<Self::Value, TlError>;

    /// What `update(bar)` would return. Changes nothing.
    fn peek(&self, bar: Self::Bar) -> Result<Self::Value, TlError>;

    /// The last committed value, or `None` before the first one.
    fn value(&self) -> Option<Self::Value>;

    /// Bars consumed so far, history included.
    fn bars_seen(&self) -> u64;
}

/// The step function shared by an indicator's batch loop and its stream.
///
/// `push` commits a bar and returns its value, or `None` while still warming
/// up. `preview` returns what the next `push` would return without touching
/// any state the next `push` reads.
pub trait SeriesStep: Clone {
    fn push(&mut self, value: f64) -> Option<f64>;
    fn preview(&self, value: f64) -> Option<f64>;
}
