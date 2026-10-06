use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_outside_day";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// A range that covers the previous one, closing beyond it.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    let outside = bar.high > previous.high && bar.low < previous.low;
    combine(
        outside && bar.close > previous.high,
        outside && bar.close < previous.low,
    )
}

#[derive(Clone, Copy, Debug)]
pub struct BarOutsideDay;

pub type BarOutsideDayStream = BarStream<BarOutsideDay, 4, 1>;

impl Kernel<4, 1> for BarOutsideDay {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_outside_day"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(params: &Params) -> State {
        BarState::new(1, 1, *params, rule)
    }
}
