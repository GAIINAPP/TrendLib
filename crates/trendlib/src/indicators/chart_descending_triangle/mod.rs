use crate::core::chart::{Lines, TrendlineState, at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_descending_triangle";

pub const PERIOD_DEFAULT: usize = 100;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const FLAT_TOL_DEFAULT: f64 = 0.02;
pub const FLAT_TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub flat_tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            flat_tol: FLAT_TOL_DEFAULT,
        }
    }
}

pub type State = TrendlineState;

/// A flat line under the swing lows, a falling one over the swing highs closing in on it, and a close below the flat one.
fn detect(lines: &Lines, close: f64, flat_tol: f64) -> f64 {
    if lines.lower.slope.abs() <= flat_tol
        && lines.upper.slope < 0.0
        && lines.converging()
        && close < lines.lower.level
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartDescendingTriangle;

pub type ChartDescendingTriangleStream = BarStream<ChartDescendingTriangle, 3, 1>;

impl Kernel<3, 1> for ChartDescendingTriangle {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_descending_triangle"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "flat_tol", params.flat_tol, FLAT_TOL_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        TrendlineState::new(params.pivot_n, params.period, params.flat_tol, detect)
    }
}
