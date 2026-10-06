use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::input::check_volume;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::PandasEwm;

pub const NAME: &str = "vzo";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    directed: PandasEwm,
    volume: PandasEwm,
    previous: Option<f64>,
}

impl State {
    fn advance(&mut self, bar: [f64; 2]) -> f64 {
        let (close, volume) = (bar[0], bar[1]);
        // The first bar has no change; finta's sign of that missing change is 0.
        let sign = match self.previous {
            Some(previous) if close > previous => 1.0,
            Some(previous) if close < previous => -1.0,
            _ => 0.0,
        };
        self.previous = Some(close);
        100.0 * (self.directed.push(sign * volume) / self.volume.push(volume))
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([self.advance(bar)])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        Some([self.clone().advance(bar)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vzo;

pub type VzoStream = BarStream<Vzo, 2, 1>;

impl Kernel<2, 1> for Vzo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["vzo"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(params: &Params) -> State {
        State {
            directed: PandasEwm::with_span(params.period),
            volume: PandasEwm::with_span(params.period),
            previous: None,
        }
    }

    fn check_inputs(inputs: &[&[f64]; 2], from: usize) -> Result<(), TlError> {
        check_volume(NAME, inputs[1], from)
    }
}
