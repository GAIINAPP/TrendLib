use crate::core::bars::{BarHistory, BarState, combine, highest_high, lowest_low};
use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_key_reversal";

pub const LOOKBACK_DEFAULT: usize = 5;
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

/// A new extreme for the window that closes back against the move.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let bar = history.back(0);
    let previous = history.back(1).close;
    let bottom = bar.low < lowest_low(history, 1, params.lookback) && bar.close > previous;
    let top = bar.high > highest_high(history, 1, params.lookback) && bar.close < previous;
    combine(bottom, top)
}

#[derive(Clone, Copy, Debug)]
pub struct BarKeyReversal;

pub type BarKeyReversalStream = BarStream<BarKeyReversal, 4, 1>;

impl Kernel<4, 1> for BarKeyReversal {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_key_reversal"];

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
