use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{CloseState, Closes};

pub const NAME: &str = "chart_v_top";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const MIN_RISE_DEFAULT: f64 = 0.05;
pub const MIN_RISE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub min_rise: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            min_rise: MIN_RISE_DEFAULT,
        }
    }
}

pub type State = CloseState<Params>;

/// A rise of `min_rise` through the first half of the window, a fall of at
/// least four fifths of that through the second, and a close back near the
/// window's first close.
fn rule(closes: &Closes, close: f64, params: &Params) -> f64 {
    let split = closes.len() - closes.len() / 2;
    let start = closes.get(0);
    let rise = (closes.max(0, split) - start) / (start + EPS);
    let fall = (closes.max(split, closes.len()) - closes.get(closes.len() - 1)) / (start + EPS);
    if rise >= params.min_rise && fall >= params.min_rise * 0.8 && close <= start * 1.02 {
        -100.0
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChartVTop;

pub type ChartVTopStream = BarStream<ChartVTop, 3, 1>;

impl Kernel<3, 1> for ChartVTop {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_v_top"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "min_rise", params.min_rise, MIN_RISE_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        CloseState::new(params.period, *params, rule)
    }
}
