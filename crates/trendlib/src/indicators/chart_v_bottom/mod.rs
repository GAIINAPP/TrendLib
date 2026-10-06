use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{CloseState, Closes};

pub const NAME: &str = "chart_v_bottom";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const MIN_DROP_DEFAULT: f64 = 0.05;
pub const MIN_DROP_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub min_drop: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            min_drop: MIN_DROP_DEFAULT,
        }
    }
}

pub type State = CloseState<Params>;

/// A fall of `min_drop` through the first half of the window, a recovery of
/// at least four fifths of that through the second, and a close back near the
/// window's first close.
fn rule(closes: &Closes, close: f64, params: &Params) -> f64 {
    let split = closes.len() - closes.len() / 2;
    let start = closes.get(0);
    let drop = (start - closes.min(0, split)) / (start + EPS);
    let recover = (closes.get(closes.len() - 1) - closes.min(split, closes.len())) / (start + EPS);
    if drop >= params.min_drop && recover >= params.min_drop * 0.8 && close >= start * 0.98 {
        100.0
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChartVBottom;

pub type ChartVBottomStream = BarStream<ChartVBottom, 3, 1>;

impl Kernel<3, 1> for ChartVBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_v_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "min_drop", params.min_drop, MIN_DROP_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        CloseState::new(params.period, *params, rule)
    }
}
