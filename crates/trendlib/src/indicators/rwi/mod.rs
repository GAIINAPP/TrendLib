use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Lagged;
use crate::indicators::atr;

pub const NAME: &str = "rwi";

pub const PERIOD_DEFAULT: usize = 14;
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
pub struct State {
    range: atr::State,
    high: Lagged,
    low: Lagged,
    root: f64,
}

impl State {
    // A zero average true range means nothing has moved since the first bar,
    // so the move it would divide is zero too.
    fn index(&self, moved: f64, average: f64) -> f64 {
        if average == 0.0 {
            0.0
        } else {
            moved / (average * self.root)
        }
    }

    fn lines(&self, bar: [f64; 3], average: f64, high: f64, low: f64) -> [f64; 2] {
        [
            self.index(bar[0] - low, average),
            self.index(high - bar[1], average),
        ]
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let average = self.range.push(bar);
        let high = self.high.push(bar[0]);
        let low = self.low.push(bar[1]);
        let ([average], high, low) = (average?, high?, low?);
        Some(self.lines(bar, average, high, low))
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let [average] = self.range.preview(bar)?;
        Some(self.lines(bar, average, self.high.earlier()?, self.low.earlier()?))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rwi;

pub type RwiStream = BarStream<Rwi, 3, 2>;

impl Kernel<3, 2> for Rwi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["rwi_high", "rwi_low"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            range: atr::Atr::state(&atr::Params {
                period: params.period,
            }),
            high: Lagged::new(params.period),
            low: Lagged::new(params.period),
            root: (params.period as f64).sqrt(),
        }
    }
}
