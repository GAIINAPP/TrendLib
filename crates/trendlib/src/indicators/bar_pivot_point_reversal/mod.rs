use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_pivot_point_reversal";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// A gap past the previous bar's extreme and a close past the open.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    let bottom = previous.close < previous.open && bar.open > previous.high && bar.close > bar.open;
    let top = previous.close > previous.open && bar.open < previous.low && bar.close < bar.open;
    combine(bottom, top)
}

#[derive(Clone, Copy, Debug)]
pub struct BarPivotPointReversal;

pub type BarPivotPointReversalStream = BarStream<BarPivotPointReversal, 4, 1>;

impl Kernel<4, 1> for BarPivotPointReversal {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_pivot_point_reversal"];

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
