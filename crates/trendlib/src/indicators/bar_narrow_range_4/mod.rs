use crate::core::bars::{BarHistory, BarState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_narrow_range_4";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// The narrowest range of the last 4 bars, this one included.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let range = |back: usize| {
        let bar = history.back(back);
        bar.high - bar.low
    };
    let narrowest = (0..4).map(range).fold(f64::INFINITY, f64::min);
    if range(0) == narrowest { 100.0 } else { 0.0 }
}

#[derive(Clone, Copy, Debug)]
pub struct BarNarrowRange4;

pub type BarNarrowRange4Stream = BarStream<BarNarrowRange4, 4, 1>;

impl Kernel<4, 1> for BarNarrowRange4 {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_narrow_range_4"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        3
    }

    fn state(params: &Params) -> State {
        BarState::new(3, 3, *params, rule)
    }
}
