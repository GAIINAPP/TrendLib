use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "pmo";

pub const FIRST_PERIOD_DEFAULT: usize = 35;
pub const FIRST_PERIOD_MIN: usize = 2;
pub const FIRST_PERIOD_MAX: usize = 100_000;
pub const SECOND_PERIOD_DEFAULT: usize = 20;
pub const SECOND_PERIOD_MIN: usize = 2;
pub const SECOND_PERIOD_MAX: usize = 100_000;
pub const SIGNAL_PERIOD_DEFAULT: usize = 10;
pub const SIGNAL_PERIOD_MIN: usize = 1;
pub const SIGNAL_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub first_period: usize,
    pub second_period: usize,
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            first_period: FIRST_PERIOD_DEFAULT,
            second_period: SECOND_PERIOD_DEFAULT,
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

// TA-Lib's arithmetic for a one-bar rate of change, ratio first. `roc` divides
// the difference instead, which rounds differently by up to 1e-10.
fn change(close: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        0.0
    } else {
        ((close / previous) - 1.0) * 100.0
    }
}

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<f64>,
    first: Ema,
    second: Ema,
    signal: Ema,
}

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let previous = self.previous.replace(bar[0])?;
        let first = self.first.push(change(bar[0], previous))?;
        let pmo = self.second.push(10.0 * first)?;
        let signal = self.signal.push(pmo)?;
        Some([pmo, signal])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let first = self.first.preview(change(bar[0], self.previous?))?;
        let pmo = self.second.preview(10.0 * first)?;
        let signal = self.signal.preview(pmo)?;
        Some([pmo, signal])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pmo;

pub type PmoStream = BarStream<Pmo, 1, 2>;

impl Kernel<1, 2> for Pmo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["pmo", "pmo_signal"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "first_period",
            params.first_period,
            FIRST_PERIOD_MIN,
            FIRST_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "second_period",
            params.second_period,
            SECOND_PERIOD_MIN,
            SECOND_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "signal_period",
            params.signal_period,
            SIGNAL_PERIOD_MIN,
            SIGNAL_PERIOD_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.first_period + params.second_period + params.signal_period - 4
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            // DecisionPoint's multiplier 2/n is an exponential average of period n - 1.
            first: Ema::new(params.first_period - 1),
            second: Ema::new(params.second_period - 1),
            signal: Ema::new(params.signal_period),
        }
    }
}
