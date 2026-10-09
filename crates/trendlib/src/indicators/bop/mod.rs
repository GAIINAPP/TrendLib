use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "bop";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

// A bar with no range gives no information about who prevailed, so it reports
// 0 rather than dividing by zero.
fn balance(bar: [f64; 4]) -> f64 {
    let (open, high, low, close) = (bar[0], bar[1], bar[2], bar[3]);
    let span = high - low;
    if span == 0.0 {
        0.0
    } else {
        (close - open) / span
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        Some([balance(bar)])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        Some([balance(bar)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bop;

pub type BopStream = BarStream<Bop, 4, 1>;

impl Kernel<4, 1> for Bop {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bop"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(_params: &Params) -> State {
        State
    }
}
