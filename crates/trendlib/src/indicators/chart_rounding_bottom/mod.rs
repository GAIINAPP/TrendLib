use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{CloseState, Closes, thirds};

pub const NAME: &str = "chart_rounding_bottom";

pub const PERIOD_DEFAULT: usize = 40;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const MIN_DEPTH_DEFAULT: f64 = 0.05;
pub const MIN_DEPTH_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub min_depth: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            min_depth: MIN_DEPTH_DEFAULT,
        }
    }
}

pub type State = CloseState<Params>;

/// A middle third lower than both outer thirds by `min_depth`, and a close
/// above the higher of them.
fn rule(closes: &Closes, close: f64, params: &Params) -> f64 {
    let (left, middle, right) = thirds(closes);
    let outer = left.min(right);
    let depth = (outer - middle) / (left + EPS);
    if middle < outer && depth >= params.min_depth && close > left.max(right) {
        100.0
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChartRoundingBottom;

pub type ChartRoundingBottomStream = BarStream<ChartRoundingBottom, 3, 1>;

impl Kernel<3, 1> for ChartRoundingBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_rounding_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "min_depth", params.min_depth, MIN_DEPTH_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        CloseState::new(params.period, *params, rule)
    }
}
