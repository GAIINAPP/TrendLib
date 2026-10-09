use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "mult";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

fn value(a: f64, b: f64) -> f64 {
    a * b
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1])])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mult;

pub type MultStream = BarStream<Mult, 2, 1>;

impl Kernel<2, 1> for Mult {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["source0", "source1"];
    const OUTPUTS: [&'static str; 1] = ["mult"];

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
