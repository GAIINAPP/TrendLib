use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::TrueRange;

pub const NAME: &str = "trange";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State(TrueRange);

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.0.push(bar[0], bar[1], bar[2]).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.0.preview(bar[0], bar[1]).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Trange;

pub type TrangeStream = BarStream<Trange, 3, 1>;

impl Kernel<3, 1> for Trange {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["trange"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(_params: &Params) -> State {
        State(TrueRange::new())
    }
}
