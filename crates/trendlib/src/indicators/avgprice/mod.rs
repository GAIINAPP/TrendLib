use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "avgprice";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

fn value(a: f64, b: f64, c: f64, d: f64) -> f64 {
    (a + b + c + d) / 4.0
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1], bar[2], bar[3])])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        Some([value(bar[0], bar[1], bar[2], bar[3])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Avgprice;

pub type AvgpriceStream = BarStream<Avgprice, 4, 1>;

impl Kernel<4, 1> for Avgprice {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["avgprice"];

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
