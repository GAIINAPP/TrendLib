use crate::core::error::TlError;
use crate::core::input::check_volume;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ad";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    total: f64,
}

/// Where the close sat in its own bar, from -1 at the low to +1 at the high,
/// multiplied by the volume that traded there.
///
/// A bar with no range says nothing about where buyers won, so it contributes
/// nothing rather than dividing by zero. TA-Lib skips such a bar too.
fn flow(bar: [f64; 4]) -> f64 {
    let (high, low, close, volume) = (bar[0], bar[1], bar[2], bar[3]);
    let span = high - low;
    if span == 0.0 {
        0.0
    } else {
        ((close - low) - (high - close)) / span * volume
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.total += flow(bar);
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        Some([self.total + flow(bar)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ad;

pub type AdStream = BarStream<Ad, 4, 1>;

impl Kernel<4, 1> for Ad {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["high", "low", "close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["ad"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(_params: &Params) -> State {
        State::default()
    }

    fn check_inputs(inputs: &[&[f64]; 4], from: usize) -> Result<(), TlError> {
        check_volume(NAME, inputs[3], from)
    }
}
