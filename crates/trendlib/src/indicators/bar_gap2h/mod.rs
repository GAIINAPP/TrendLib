use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_gap2h";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// An open beyond the extremes of both bars before it.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let bar = history.back(0);
    let (one, two) = (history.back(1), history.back(2));
    combine(
        bar.open > one.high.max(two.high),
        bar.open < one.low.min(two.low),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct BarGap2h;

pub type BarGap2hStream = BarStream<BarGap2h, 4, 1>;

impl Kernel<4, 1> for BarGap2h {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_gap2h"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        2
    }

    fn state(params: &Params) -> State {
        BarState::new(2, 2, *params, rule)
    }
}
