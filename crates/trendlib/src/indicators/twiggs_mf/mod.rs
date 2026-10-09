use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::input::check_volume;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Wilder;

pub const NAME: &str = "twiggs_mf";

pub const PERIOD_DEFAULT: usize = 21;
pub const PERIOD_MIN: usize = 1;
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
    previous: Option<f64>,
    flow: Wilder,
    volume: Wilder,
}

impl State {
    fn advance(&mut self, [high, low, close, volume]: [f64; 4]) -> Option<f64> {
        let previous = self.previous.replace(close)?;
        let top = high.max(previous);
        let bottom = low.min(previous);
        let range = top - bottom;
        let flow = if range == 0.0 {
            0.0
        } else {
            ((close - bottom) - (top - close)) / range * volume
        };
        let flow = self.flow.push(flow);
        let volume = self.volume.push(volume);
        let (flow, volume) = (flow?, volume?);
        Some(if volume == 0.0 { 0.0 } else { flow / volume })
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TwiggsMf;

pub type TwiggsMfStream = BarStream<TwiggsMf, 4, 1>;

impl Kernel<4, 1> for TwiggsMf {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["high", "low", "close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["twiggs_mf"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            flow: Wilder::new(params.period),
            volume: Wilder::new(params.period),
        }
    }

    fn check_inputs(inputs: &[&[f64]; 4], from: usize) -> Result<(), TlError> {
        check_volume(NAME, inputs[3], from)
    }
}
