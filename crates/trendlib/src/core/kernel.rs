//! The batch loop and the stream, shared by every indicator.
//!
//! An indicator supplies a [`Kernel`]: its parameters, its warm-up length and a
//! [`Step`] that turns one bar into one row of outputs. Everything else here is
//! derived from that, so batch and stream cannot drift apart - they call the
//! same step function in the same order.
//!
//! Inputs arrive the way NumPy hands them over, one contiguous column per input,
//! and bars are assembled on the fly. Nothing is transposed and nothing is
//! allocated per bar.

use crate::core::error::TlError;
use crate::core::input::{Prepared, SeriesInput, prepare};
use crate::core::output::nan_filled;
use crate::core::traits::Stream;

/// One bar in, one row of outputs out.
///
/// `push` commits a bar and returns its outputs, or `None` while the indicator
/// is still warming up. `preview` returns what the next `push` would return and
/// must not touch any state that `push` reads.
///
/// Integer outputs travel as `f64` here and are narrowed at the boundary. Every
/// integer an indicator produces - pattern flags, trend direction, bar indices -
/// is far inside the 2^53 a float64 represents exactly, so nothing is lost.
pub trait Step<const I: usize, const O: usize>: Clone {
    fn push(&mut self, bar: [f64; I]) -> Option<[f64; O]>;
    fn preview(&self, bar: [f64; I]) -> Option<[f64; O]>;

    /// The row the first bar will come from, given once before any `push`.
    ///
    /// Only indicators that report a row index need this. Leading warm-up rows
    /// are skipped, so without it such an indicator would count from the first
    /// valid bar and report an index into a series the caller never passed.
    fn start_at(&mut self, _row: usize) {}
}

/// What an indicator has to supply. The rest of its public surface is derived.
pub trait Kernel<const I: usize, const O: usize>: Sized {
    const NAME: &'static str;
    /// Input names in declaration order, which is also Python's positional order.
    const INPUTS: [&'static str; I];
    /// Output names in declaration order, which is also the returned tuple order.
    const OUTPUTS: [&'static str; O];

    type Params: Clone;
    type State: Step<I, O>;

    fn validate(params: &Self::Params) -> Result<(), TlError>;
    fn lookback(params: &Self::Params) -> usize;
    fn state(params: &Self::Params) -> Self::State;

    /// Checks beyond "every input is finite", such as rejecting a negative
    /// volume. Runs after the first valid bar has been located.
    fn check_inputs(_inputs: &[&[f64]; I], _from: usize) -> Result<(), TlError> {
        Ok(())
    }

    /// Run over the whole input, aligned to it, warm-up rows `NaN`.
    fn batch(inputs: [&[f64]; I], params: &Self::Params) -> Result<[Vec<f64>; O], TlError> {
        batch::<Self, I, O>(inputs, params)
    }

    /// The batch output and a stream positioned at its last bar, in one pass.
    fn open_and_fill(
        inputs: [&[f64]; I],
        params: &Self::Params,
    ) -> Result<(BarStream<Self, I, O>, [Vec<f64>; O]), TlError> {
        open_and_fill::<Self, I, O>(inputs, params)
    }
}

fn validated<K: Kernel<I, O>, const I: usize, const O: usize>(
    inputs: &[&[f64]; I],
    params: &K::Params,
) -> Result<Prepared, TlError> {
    K::validate(params)?;
    let named: [SeriesInput<'_>; I] =
        std::array::from_fn(|i| SeriesInput::new(K::INPUTS[i], inputs[i]));
    let prepared = prepare(K::NAME, &named)?;
    K::check_inputs(inputs, prepared.first_valid)?;
    Ok(prepared)
}

fn bar_at<const I: usize>(inputs: &[&[f64]; I], row: usize) -> [f64; I] {
    std::array::from_fn(|i| inputs[i][row])
}

/// Run an indicator over the whole input, aligned to it, warm-up rows `NaN`.
pub fn batch<K: Kernel<I, O>, const I: usize, const O: usize>(
    inputs: [&[f64]; I],
    params: &K::Params,
) -> Result<[Vec<f64>; O], TlError> {
    let prepared = validated::<K, I, O>(&inputs, params)?;
    let mut out: [Vec<f64>; O] = std::array::from_fn(|_| nan_filled(prepared.len));
    let mut state = K::state(params);
    state.start_at(prepared.first_valid);
    for row in prepared.first_valid..prepared.len {
        if let Some(values) = state.push(bar_at(&inputs, row)) {
            for (column, value) in out.iter_mut().zip(values) {
                column[row] = value;
            }
        }
    }
    Ok(out)
}

/// The batch output and a stream positioned at its last bar, in one pass.
pub fn open_and_fill<K: Kernel<I, O>, const I: usize, const O: usize>(
    inputs: [&[f64]; I],
    params: &K::Params,
) -> Result<(BarStream<K, I, O>, [Vec<f64>; O]), TlError> {
    let prepared = validated::<K, I, O>(&inputs, params)?;
    let needed = K::lookback(params) + 1;
    if prepared.valid_len() < needed {
        return Err(TlError::insufficient_history(
            K::NAME,
            needed,
            prepared.valid_len(),
        ));
    }

    let mut out: [Vec<f64>; O] = std::array::from_fn(|_| nan_filled(prepared.len));
    let mut state = K::state(params);
    state.start_at(prepared.first_valid);
    let mut last = None;
    for row in prepared.first_valid..prepared.len {
        if let Some(values) = state.push(bar_at(&inputs, row)) {
            for (column, value) in out.iter_mut().zip(values) {
                column[row] = value;
            }
            last = Some(values);
        }
    }

    let stream = BarStream {
        params: params.clone(),
        state,
        last,
        bars_seen: prepared.len as u64,
    };
    Ok((stream, out))
}

/// A live indicator value.
#[derive(Debug)]
pub struct BarStream<K: Kernel<I, O>, const I: usize, const O: usize> {
    params: K::Params,
    state: K::State,
    last: Option<[f64; O]>,
    bars_seen: u64,
}

impl<K: Kernel<I, O>, const I: usize, const O: usize> Clone for BarStream<K, I, O> {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            state: self.state.clone(),
            last: self.last,
            bars_seen: self.bars_seen,
        }
    }
}

impl<K: Kernel<I, O>, const I: usize, const O: usize> BarStream<K, I, O> {
    pub fn params(&self) -> &K::Params {
        &self.params
    }

    fn check_bar(&self, bar: [f64; I]) -> Result<(), TlError> {
        for (index, value) in bar.iter().enumerate() {
            if !value.is_finite() {
                return Err(TlError::non_finite(
                    K::NAME,
                    K::INPUTS[index],
                    self.bars_seen as usize,
                    *value,
                ));
            }
        }
        K::check_inputs(&std::array::from_fn(|i| &bar[i..i + 1]), 0)
    }
}

impl<K: Kernel<I, O>, const I: usize, const O: usize> Stream for BarStream<K, I, O> {
    type Params = K::Params;
    type History<'a> = [&'a [f64]; I];
    type Bar = [f64; I];
    type Value = [f64; O];

    fn open(history: Self::History<'_>, params: &K::Params) -> Result<Self, TlError> {
        open_and_fill::<K, I, O>(history, params).map(|(stream, _)| stream)
    }

    fn update(&mut self, bar: Self::Bar) -> Result<Self::Value, TlError> {
        self.check_bar(bar)?;
        let value = self
            .state
            .push(bar)
            .expect("a stream past its lookback always produces a value");
        self.last = Some(value);
        self.bars_seen += 1;
        Ok(value)
    }

    fn peek(&self, bar: Self::Bar) -> Result<Self::Value, TlError> {
        self.check_bar(bar)?;
        Ok(self
            .state
            .preview(bar)
            .expect("a stream past its lookback always produces a value"))
    }

    fn value(&self) -> Option<Self::Value> {
        self.last
    }

    fn bars_seen(&self) -> u64 {
        self.bars_seen
    }
}
