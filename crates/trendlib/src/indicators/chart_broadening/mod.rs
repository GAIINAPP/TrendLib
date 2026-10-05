use crate::core::chart::{Lines, TrendlineState, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_broadening";

pub const PERIOD_DEFAULT: usize = 80;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
        }
    }
}

pub type State = TrendlineState;

/// A rising line over the swing highs, a falling one under the swing lows, and a close through either.
fn detect(lines: &Lines, close: f64, _tolerance: f64) -> f64 {
    if lines.upper.slope > 0.0 && lines.lower.slope < 0.0 {
        lines.breakout(close)
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartBroadening;

pub type ChartBroadeningStream = BarStream<ChartBroadening, 3, 1>;

impl Kernel<3, 1> for ChartBroadening {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_broadening"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        TrendlineState::new(params.pivot_n, params.period, 0.0, detect)
    }
}
