use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingMean;

pub const NAME: &str = "envelope";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PERCENT_DEFAULT: f64 = 10.0;
pub const PERCENT_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub percent: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            percent: PERCENT_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    average: RollingMean,
    up: f64,
    down: f64,
}

impl State {
    fn bands(&self, mean: f64) -> [f64; 3] {
        [mean * self.up, mean, mean * self.down]
    }
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        let mean = self.average.push(bar[0])?;
        Some(self.bands(mean))
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 3]> {
        self.average.preview(bar[0]).map(|mean| self.bands(mean))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Envelope;

pub type EnvelopeStream = BarStream<Envelope, 1, 3>;

impl Kernel<1, 3> for Envelope {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = ["envelope_upper", "envelope_middle", "envelope_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "percent", params.percent, PERCENT_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period - 1
    }

    fn state(params: &Params) -> State {
        State {
            average: RollingMean::new(params.period),
            up: 1.0 + params.percent / 100.0,
            down: 1.0 - params.percent / 100.0,
        }
    }
}
