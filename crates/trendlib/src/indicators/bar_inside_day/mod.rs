use crate::core::bars::{BarHistory, BarState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_inside_day";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// A range inside the previous bar's on both sides.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    if bar.high < previous.high && bar.low > previous.low {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarInsideDay;

pub type BarInsideDayStream = BarStream<BarInsideDay, 4, 1>;

impl Kernel<4, 1> for BarInsideDay {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_inside_day"];

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
