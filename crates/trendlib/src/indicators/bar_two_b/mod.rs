use crate::core::bars::{BarHistory, BarState, combine, highest_high, lowest_low};
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_two_b";

pub const LOOKBACK_DEFAULT: usize = 5;
pub const LOOKBACK_MIN: usize = 1;
pub const LOOKBACK_MAX: usize = 100_000;
pub const TOL_DEFAULT: f64 = 0.01;
pub const TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub lookback: usize,
    pub tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            lookback: LOOKBACK_DEFAULT,
            tol: TOL_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// A bar beyond the extreme of the `lookback` bars before it, then a close back
/// inside that extreme.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let breakout = history.back(1);
    let close = history.back(0).close;
    let prior_high = highest_high(history, 2, params.lookback);
    let prior_low = lowest_low(history, 2, params.lookback);
    let failed_high = breakout.high > prior_high * (1.0 + params.tol) && close < prior_high;
    let failed_low = breakout.low < prior_low * (1.0 - params.tol) && close > prior_low;
    combine(failed_low, failed_high)
}

#[derive(Clone, Copy, Debug)]
pub struct BarTwoB;

pub type BarTwoBStream = BarStream<BarTwoB, 4, 1>;

impl Kernel<4, 1> for BarTwoB {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_two_b"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "lookback",
            params.lookback,
            LOOKBACK_MIN,
            LOOKBACK_MAX,
        )?;
        at_least(NAME, "tol", params.tol, TOL_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.lookback + 1
    }

    fn state(params: &Params) -> State {
        BarState::new(params.lookback + 1, params.lookback + 1, *params, rule)
    }
}
