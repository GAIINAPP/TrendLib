use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "nvi";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<(f64, f64)>,
    index: f64,
}

/// The index starts at 1000 so that its moves read as percentages of where it
/// began. TA-Lib uses the same figure.
pub const SEED: f64 = 1000.0;

fn step(index: f64, close: f64, previous: (f64, f64), volume: f64) -> f64 {
    let (previous_close, previous_volume) = previous;
    if volume < previous_volume {
        index + index * (close - previous_close) / previous_close
    } else {
        index
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        if let Some(previous) = self.previous.replace((bar[0], bar[1])) {
            self.index = step(self.index, bar[0], previous, bar[1]);
        }
        Some([self.index])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([match self.previous {
            Some(previous) => step(self.index, bar[0], previous, bar[1]),
            None => self.index,
        }])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Nvi;

pub type NviStream = BarStream<Nvi, 2, 1>;

impl Kernel<2, 1> for Nvi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["nvi"];

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
            previous: None,
            index: SEED,
        }
    }
}
