use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_fakey";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// An inside bar, then a close beyond the bar it was inside.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, inner, mother) = (history.back(0), history.back(1), history.back(2));
    let inside = inner.high < mother.high && inner.low > mother.low;
    combine(
        inside && bar.close > mother.high,
        inside && bar.close < mother.low,
    )
}

#[derive(Clone, Copy, Debug)]
pub struct BarFakey;

pub type BarFakeyStream = BarStream<BarFakey, 4, 1>;

impl Kernel<4, 1> for BarFakey {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_fakey"];

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
