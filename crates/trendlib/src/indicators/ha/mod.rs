use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ha";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params;

impl Default for Params {
    fn default() -> Self {
        Self
    }
}

#[derive(Clone, Debug)]
pub struct State {
    /// The previous candle's open and close, which the next open is built from.
    previous: Option<(f64, f64)>,
}

fn candle(previous: Option<(f64, f64)>, bar: [f64; 4]) -> [f64; 4] {
    let [open, high, low, close] = bar;
    let ha_close = (open + high + low + close) / 4.0;
    // The first candle has no previous one to carry, so it opens at the
    // midpoint of the bar itself.
    let ha_open = match previous {
        Some((prior_open, prior_close)) => (prior_open + prior_close) / 2.0,
        None => (open + close) / 2.0,
    };
    [
        ha_open,
        high.max(ha_open).max(ha_close),
        low.min(ha_open).min(ha_close),
        ha_close,
    ]
}

impl Step<4, 4> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 4]> {
        let out = candle(self.previous, bar);
        self.previous = Some((out[0], out[3]));
        Some(out)
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 4]> {
        Some(candle(self.previous, bar))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ha;

pub type HaStream = BarStream<Ha, 4, 4>;

impl Kernel<4, 4> for Ha {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 4] = ["ha_open", "ha_high", "ha_low", "ha_close"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(_params: &Params) -> State {
        State { previous: None }
    }
}
