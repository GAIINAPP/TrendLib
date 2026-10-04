use crate::core::error::TlError;
use crate::core::math::RollingMean;
use crate::core::single::{self, SingleSeries, SingleStream};
use crate::core::traits::{Indicator, SeriesStep};

pub const NAME: &str = "sma";

// Mirrors spec.yaml. From M2 `cargo xtask generate` owns these three values and
// the validate body below; a Python test asserts they agree until then.
pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State(RollingMean);

impl SeriesStep for State {
    fn push(&mut self, value: f64) -> Option<f64> {
        self.0.push(value)
    }

    fn preview(&self, value: f64) -> Option<f64> {
        self.0.preview(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sma;

pub type SmaStream = SingleStream<Sma>;

impl SingleSeries for Sma {
    const NAME: &'static str = NAME;

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State(RollingMean::new(params.period))
    }
}

impl Indicator for Sma {
    const NAME: &'static str = NAME;

    type Params = Params;
    type Input<'a> = &'a [f64];
    type Output = Vec<f64>;
    type Stream = SmaStream;

    fn validate(params: &Params) -> Result<(), TlError> {
        <Self as SingleSeries>::validate(params)
    }

    fn lookback(params: &Params) -> usize {
        <Self as SingleSeries>::lookback(params)
    }

    fn batch(source: &[f64], params: &Params) -> Result<Vec<f64>, TlError> {
        single::batch::<Self>(source, params)
    }

    fn open_and_fill(source: &[f64], params: &Params) -> Result<(SmaStream, Vec<f64>), TlError> {
        single::open_and_fill::<Self>(source, params)
    }
}
