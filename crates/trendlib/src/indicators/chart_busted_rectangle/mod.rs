use crate::core::chart::{Lines, TrendlineRule, TrendlineState};
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{BustPair, BustState};
use crate::indicators::chart_rectangle::{FLAT_TOL_DEFAULT, PERIOD_DEFAULT, PIVOT_N_DEFAULT};

pub const NAME: &str = "chart_busted_rectangle";

pub const REVERSAL_BARS_DEFAULT: usize = 10;
pub const REVERSAL_BARS_MIN: usize = 1;
pub const REVERSAL_BARS_MAX: usize = 100_000;
pub const REVERSAL_PCT_DEFAULT: f64 = 0.05;
pub const REVERSAL_PCT_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub reversal_bars: usize,
    pub reversal_pct: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            reversal_bars: REVERSAL_BARS_DEFAULT,
            reversal_pct: REVERSAL_PCT_DEFAULT,
        }
    }
}

pub type State = BustPair<TrendlineState, TrendlineState>;

/// The rectangle's upward reading alone: the oracle's `rectangle_bottom`.
fn upward(lines: &Lines, close: f64, flat_tol: f64) -> f64 {
    let flat = lines.upper.slope.abs() <= flat_tol && lines.lower.slope.abs() <= flat_tol;
    if flat && close > lines.upper.level {
        100.0
    } else {
        0.0
    }
}

/// The rectangle's downward reading alone: the oracle's `rectangle_top`.
fn downward(lines: &Lines, close: f64, flat_tol: f64) -> f64 {
    let flat = lines.upper.slope.abs() <= flat_tol && lines.lower.slope.abs() <= flat_tol;
    if flat && close < lines.lower.level {
        -100.0
    } else {
        0.0
    }
}

fn rectangle(rule: TrendlineRule) -> TrendlineState {
    TrendlineState::new(PIVOT_N_DEFAULT, PERIOD_DEFAULT, FLAT_TOL_DEFAULT, rule)
}

#[derive(Clone, Copy, Debug)]
pub struct ChartBustedRectangle;

pub type ChartBustedRectangleStream = BarStream<ChartBustedRectangle, 3, 1>;

impl Kernel<3, 1> for ChartBustedRectangle {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_busted_rectangle"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "reversal_bars",
            params.reversal_bars,
            REVERSAL_BARS_MIN,
            REVERSAL_BARS_MAX,
        )?;
        at_least(NAME, "reversal_pct", params.reversal_pct, REVERSAL_PCT_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        PERIOD_DEFAULT
    }

    fn state(params: &Params) -> State {
        BustPair {
            first: BustState::new(
                rectangle(upward),
                true,
                params.reversal_bars,
                params.reversal_pct,
                PERIOD_DEFAULT,
            ),
            second: BustState::new(
                rectangle(downward),
                false,
                params.reversal_bars,
                params.reversal_pct,
                PERIOD_DEFAULT,
            ),
        }
    }
}
