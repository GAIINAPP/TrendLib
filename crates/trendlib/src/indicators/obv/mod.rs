use crate::core::error::TlError;
use crate::core::input::check_volume;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "obv";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    previous_close: Option<f64>,
    total: f64,
}

impl State {
    fn advance(total: f64, previous: Option<f64>, close: f64, volume: f64) -> f64 {
        match previous {
            // The first bar seeds the running total with its own volume, so
            // the line starts where the first day's trading put it.
            None => volume,
            Some(previous) if close > previous => total + volume,
            Some(previous) if close < previous => total - volume,
            Some(_) => total,
        }
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.total = Self::advance(self.total, self.previous_close, bar[0], bar[1]);
        self.previous_close = Some(bar[0]);
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([Self::advance(
            self.total,
            self.previous_close,
            bar[0],
            bar[1],
        )])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Obv;

pub type ObvStream = BarStream<Obv, 2, 1>;

impl Kernel<2, 1> for Obv {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["obv"];

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

    fn check_inputs(inputs: &[&[f64]; 2], from: usize) -> Result<(), TlError> {
        check_volume(NAME, inputs[1], from)
    }
}
