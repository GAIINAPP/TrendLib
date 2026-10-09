use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "wad";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous_close: Option<f64>,
    total: f64,
}

/// What the bar added: how far it closed from the lower of its own low and the
/// previous close when it rose, from the higher of its high and that close when
/// it fell, and nothing when it finished where it started.
fn step(high: f64, low: f64, close: f64, previous: f64) -> f64 {
    if close > previous {
        close - low.min(previous)
    } else if close < previous {
        close - high.max(previous)
    } else {
        0.0
    }
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        if let Some(previous) = self.previous_close.replace(bar[2]) {
            self.total += step(bar[0], bar[1], bar[2], previous);
        }
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let added = match self.previous_close {
            Some(previous) => step(bar[0], bar[1], bar[2], previous),
            None => 0.0,
        };
        Some([self.total + added])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Wad;

pub type WadStream = BarStream<Wad, 3, 1>;

impl Kernel<3, 1> for Wad {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["wad"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(_params: &Params) -> State {
        State {
            previous_close: None,
            total: 0.0,
        }
    }
}
