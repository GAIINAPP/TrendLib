use crate::core::bars::{BarHistory, BarState, combine, highest_high, lowest_low};
use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_one_day_reversal";

pub const LOOKBACK_DEFAULT: usize = 10;
pub const LOOKBACK_MIN: usize = 1;
pub const LOOKBACK_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub lookback: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            lookback: LOOKBACK_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// A new extreme for the window, closing in the other half of the bar.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let bar = history.back(0);
    let middle = (bar.high + bar.low) / 2.0;
    let bottom = bar.low < lowest_low(history, 1, params.lookback) && bar.close > middle;
    let top = bar.high > highest_high(history, 1, params.lookback) && bar.close < middle;
    combine(bottom, top)
}

#[derive(Clone, Copy, Debug)]
pub struct BarOneDayReversal;

pub type BarOneDayReversalStream = BarStream<BarOneDayReversal, 4, 1>;

impl Kernel<4, 1> for BarOneDayReversal {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_one_day_reversal"];

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
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.lookback
    }

    fn state(params: &Params) -> State {
        BarState::new(params.lookback, params.lookback, *params, rule)
    }
}
