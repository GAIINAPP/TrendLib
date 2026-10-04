use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ln";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

fn value(a: f64) -> f64 {
    a.ln()
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        Some([value(bar[0])])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        Some([value(bar[0])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ln;

pub type LnStream = BarStream<Ln, 1, 1>;

impl Kernel<1, 1> for Ln {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ln"];

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
