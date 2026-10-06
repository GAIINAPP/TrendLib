use crate::core::chart::{EPS, Lines, TrendlineState, at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_ascending_channel";

pub const PERIOD_DEFAULT: usize = 80;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 4;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const PARALLEL_TOL_DEFAULT: f64 = 0.25;
pub const PARALLEL_TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub parallel_tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            parallel_tol: PARALLEL_TOL_DEFAULT,
        }
    }
}

pub type State = TrendlineState;

/// The two slopes differ by at most `parallel_tol` of the lower line's.
fn parallel(lines: &Lines, parallel_tol: f64) -> bool {
    (lines.upper.slope - lines.lower.slope).abs() / (lines.lower.slope.abs() + EPS) <= parallel_tol
}

/// Two rising lines of about the same slope, and a close below the lower one.
fn detect(lines: &Lines, close: f64, parallel_tol: f64) -> f64 {
    if lines.upper.slope > 0.0
        && lines.lower.slope > 0.0
        && parallel(lines, parallel_tol)
        && close < lines.lower.level
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartAscendingChannel;

pub type ChartAscendingChannelStream = BarStream<ChartAscendingChannel, 3, 1>;

impl Kernel<3, 1> for ChartAscendingChannel {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_ascending_channel"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "parallel_tol", params.parallel_tol, PARALLEL_TOL_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        TrendlineState::new(params.pivot_n, params.period, params.parallel_tol, detect)
    }
}
