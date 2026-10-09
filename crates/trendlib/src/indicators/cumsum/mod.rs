use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "cumsum";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    total: f64,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.total += bar[0];
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        Some([self.total + bar[0]])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cumsum;

pub type CumsumStream = BarStream<Cumsum, 1, 1>;

impl Kernel<1, 1> for Cumsum {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["cumsum"];

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
}
