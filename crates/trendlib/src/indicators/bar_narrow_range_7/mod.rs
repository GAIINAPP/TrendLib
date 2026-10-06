use crate::core::bars::{BarHistory, BarState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_narrow_range_7";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// The narrowest range of the last 7 bars, this one included.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let range = |back: usize| {
        let bar = history.back(back);
        bar.high - bar.low
    };
    let narrowest = (0..7).map(range).fold(f64::INFINITY, f64::min);
    if range(0) == narrowest { 100.0 } else { 0.0 }
}

#[derive(Clone, Copy, Debug)]
pub struct BarNarrowRange7;

pub type BarNarrowRange7Stream = BarStream<BarNarrowRange7, 4, 1>;

impl Kernel<4, 1> for BarNarrowRange7 {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_narrow_range_7"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        6
    }

    fn state(params: &Params) -> State {
        BarState::new(6, 6, *params, rule)
    }
}
