use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "cpr";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    previous: Option<[f64; 3]>,
}

fn levels(bar: [f64; 3]) -> [f64; 3] {
    let [high, low, close] = bar;
    let pivot = (high + low + close) / 3.0;
    let bottom = (high + low) / 2.0;
    [pivot, bottom, 2.0 * pivot - bottom]
}

impl Step<3, 3> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 3]> {
        self.previous.replace(bar).map(levels)
    }

    fn preview(&self, _bar: [f64; 3]) -> Option<[f64; 3]> {
        self.previous.map(levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cpr;

pub type CprStream = BarStream<Cpr, 3, 3>;

impl Kernel<3, 3> for Cpr {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 3] = ["cpr_pivot", "cpr_bc", "cpr_tc"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(_params: &Params) -> State {
        State { previous: None }
    }
}
