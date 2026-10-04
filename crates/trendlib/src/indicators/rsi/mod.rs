use crate::core::error::TlError;
use crate::core::math::Wilder;
use crate::core::single::{self, SingleSeries, SingleStream};
use crate::core::traits::{Indicator, SeriesStep};

pub const NAME: &str = "rsi";

// Mirrors spec.yaml. From M2 `cargo xtask generate` owns these three values and
// the validate body below; a Python test asserts they agree until then.
const PERIOD_DEFAULT: usize = 14;
const PERIOD_MIN: usize = 2;
const PERIOD_MAX: usize = 100_000;

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
pub struct State {
    previous: Option<f64>,
    gain: Wilder,
    loss: Wilder,
}

impl State {
    fn split(change: f64) -> (f64, f64) {
        if change < 0.0 {
            (0.0, -change)
        } else {
            (change, 0.0)
        }
    }

    // A flat stretch leaves both averages at zero. TA-Lib reports 0 there, and
    // so does TrendLib; any other choice would divide by zero.
    fn index(gain: f64, loss: f64) -> f64 {
        let total = gain + loss;
        if total == 0.0 {
            0.0
        } else {
            100.0 * gain / total
        }
    }
}

impl SeriesStep for State {
    fn push(&mut self, value: f64) -> Option<f64> {
        let previous = self.previous.replace(value)?;
        let (gain, loss) = Self::split(value - previous);
        match (self.gain.push(gain), self.loss.push(loss)) {
            (Some(gain), Some(loss)) => Some(Self::index(gain, loss)),
            _ => None,
        }
    }

    fn preview(&self, value: f64) -> Option<f64> {
        let previous = self.previous?;
        let (gain, loss) = Self::split(value - previous);
        match (self.gain.preview(gain), self.loss.preview(loss)) {
            (Some(gain), Some(loss)) => Some(Self::index(gain, loss)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rsi;

pub type RsiStream = SingleStream<Rsi>;

impl SingleSeries for Rsi {
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
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            gain: Wilder::new(params.period),
            loss: Wilder::new(params.period),
        }
    }
}

impl Indicator for Rsi {
    const NAME: &'static str = NAME;

    type Params = Params;
    type Input<'a> = &'a [f64];
    type Output = Vec<f64>;
    type Stream = RsiStream;

    fn validate(params: &Params) -> Result<(), TlError> {
        <Self as SingleSeries>::validate(params)
    }

    fn lookback(params: &Params) -> usize {
        <Self as SingleSeries>::lookback(params)
    }

    fn batch(source: &[f64], params: &Params) -> Result<Vec<f64>, TlError> {
        single::batch::<Self>(source, params)
    }

    fn open_and_fill(source: &[f64], params: &Params) -> Result<(RsiStream, Vec<f64>), TlError> {
        single::open_and_fill::<Self>(source, params)
    }
}
