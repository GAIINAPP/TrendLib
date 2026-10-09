use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_hook_reversal";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// An open inside the previous bar's range and a close past its open.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    let opens_inside = bar.open > previous.low && bar.open < previous.high;
    let bottom = previous.close < previous.open && opens_inside && bar.close > previous.open;
    let top = previous.close > previous.open && opens_inside && bar.close < previous.open;
    combine(bottom, top)
}

#[derive(Clone, Copy, Debug)]
pub struct BarHookReversal;

pub type BarHookReversalStream = BarStream<BarHookReversal, 4, 1>;

impl Kernel<4, 1> for BarHookReversal {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_hook_reversal"];

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
