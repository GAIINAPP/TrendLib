use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{CloseState, Closes, thirds};

pub const NAME: &str = "chart_descending_scallop";

pub const PERIOD_DEFAULT: usize = 25;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub type State = CloseState<Params>;

/// A ∩ over the window, price lower than at its
/// start, and a close below every close in it.
fn rule(closes: &Closes, close: f64, _params: &Params) -> f64 {
    let (left, middle, right) = thirds(closes);
    if middle > left
        && right < middle
        && right <= left * 0.95
        && close < closes.get(0)
        && close < closes.min(0, closes.len())
    {
        -100.0
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChartDescendingScallop;

pub type ChartDescendingScallopStream = BarStream<ChartDescendingScallop, 3, 1>;

impl Kernel<3, 1> for ChartDescendingScallop {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_descending_scallop"];

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
        CloseState::new(params.period, *params, rule)
    }
}
