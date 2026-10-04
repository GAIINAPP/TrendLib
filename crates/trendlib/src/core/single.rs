//! The batch loop and stream shared by every single-series, single-output
//! indicator. Both drive the same [`SeriesStep`], so stream output equals batch
//! output bitwise by construction (`CLAUDE.md` rule 6).

use crate::core::error::TlError;
use crate::core::input::{SeriesInput, prepare};
use crate::core::output::nan_filled;
use crate::core::traits::{SeriesStep, Stream};

/// What a single-series indicator has to supply for the shared machinery.
pub trait SingleSeries: Sized {
    const NAME: &'static str;
    const INPUT: &'static str = "source";

    type Params: Clone;
    type State: SeriesStep;

    fn validate(params: &Self::Params) -> Result<(), TlError>;
    fn lookback(params: &Self::Params) -> usize;
    fn state(params: &Self::Params) -> Self::State;
}

pub fn batch<I: SingleSeries>(source: &[f64], params: &I::Params) -> Result<Vec<f64>, TlError> {
    I::validate(params)?;
    let prepared = prepare(I::NAME, &[SeriesInput::new(I::INPUT, source)])?;
    let mut out = nan_filled(prepared.len);
    let mut state = I::state(params);
    for (row, &value) in source.iter().enumerate().skip(prepared.first_valid) {
        if let Some(computed) = state.push(value) {
            out[row] = computed;
        }
    }
    Ok(out)
}

pub fn open_and_fill<I: SingleSeries>(
    source: &[f64],
    params: &I::Params,
) -> Result<(SingleStream<I>, Vec<f64>), TlError> {
    I::validate(params)?;
    let prepared = prepare(I::NAME, &[SeriesInput::new(I::INPUT, source)])?;
    let needed = I::lookback(params) + 1;
    if prepared.valid_len() < needed {
        return Err(TlError::insufficient_history(
            I::NAME,
            needed,
            prepared.valid_len(),
        ));
    }

    let mut out = nan_filled(prepared.len);
    let mut state = I::state(params);
    let mut last = None;
    for (row, &value) in source.iter().enumerate().skip(prepared.first_valid) {
        if let Some(computed) = state.push(value) {
            out[row] = computed;
            last = Some(computed);
        }
    }

    let stream = SingleStream {
        params: params.clone(),
        state,
        last,
        bars_seen: source.len() as u64,
    };
    Ok((stream, out))
}

/// A live value of a single-series indicator.
#[derive(Debug)]
pub struct SingleStream<I: SingleSeries> {
    params: I::Params,
    state: I::State,
    last: Option<f64>,
    bars_seen: u64,
}

impl<I: SingleSeries> Clone for SingleStream<I> {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            state: self.state.clone(),
            last: self.last,
            bars_seen: self.bars_seen,
        }
    }
}

impl<I: SingleSeries> SingleStream<I> {
    pub fn params(&self) -> &I::Params {
        &self.params
    }

    fn check_bar(&self, bar: f64) -> Result<(), TlError> {
        if bar.is_finite() {
            return Ok(());
        }
        Err(TlError::non_finite(
            I::NAME,
            I::INPUT,
            self.bars_seen as usize,
            bar,
        ))
    }
}

impl<I: SingleSeries> Stream for SingleStream<I> {
    type Params = I::Params;
    type Bar = f64;
    type Value = f64;

    fn open(history: &[f64], params: &I::Params) -> Result<Self, TlError> {
        open_and_fill::<I>(history, params).map(|(stream, _)| stream)
    }

    fn update(&mut self, bar: f64) -> Result<f64, TlError> {
        self.check_bar(bar)?;
        let value = self
            .state
            .push(bar)
            .expect("a stream past its lookback always produces a value");
        self.last = Some(value);
        self.bars_seen += 1;
        Ok(value)
    }

    fn peek(&self, bar: f64) -> Result<f64, TlError> {
        self.check_bar(bar)?;
        Ok(self
            .state
            .preview(bar)
            .expect("a stream past its lookback always produces a value"))
    }

    fn value(&self) -> Option<f64> {
        self.last
    }

    fn bars_seen(&self) -> u64 {
        self.bars_seen
    }
}
