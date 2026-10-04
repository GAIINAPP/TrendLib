use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pvt";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<f64>,
    total: f64,
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        if let Some(previous) = self.previous.replace(bar[0]) {
            self.total += (bar[0] - previous) / previous * bar[1];
        }
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let step = match self.previous {
            Some(previous) => (bar[0] - previous) / previous * bar[1],
            None => 0.0,
        };
        Some([self.total + step])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pvt;

pub type PvtStream = BarStream<Pvt, 2, 1>;

impl Kernel<2, 1> for Pvt {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["pvt"];

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
            total: 0.0,
        }
    }
}
