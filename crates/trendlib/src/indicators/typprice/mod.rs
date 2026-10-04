use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "typprice";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

fn value(a: f64, b: f64, c: f64) -> f64 {
    (a + b + c) / 3.0
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1], bar[2])])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1], bar[2])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Typprice;

pub type TyppriceStream = BarStream<Typprice, 3, 1>;

impl Kernel<3, 1> for Typprice {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["typprice"];

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
