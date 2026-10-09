use crate::core::chart::{Pole, PoleState, at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_bear_pennant";

pub const PERIOD_DEFAULT: usize = 15;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 3;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const POLE_BARS_DEFAULT: usize = 10;
pub const POLE_BARS_MIN: usize = 1;
pub const POLE_BARS_MAX: usize = 100_000;
pub const MIN_POLE_DEFAULT: f64 = 0.05;
pub const MIN_POLE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub pole_bars: usize,
    pub min_pole: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            pole_bars: POLE_BARS_DEFAULT,
            min_pole: MIN_POLE_DEFAULT,
        }
    }
}

pub type State = PoleState;

/// A fall, then a falling line over the swing highs and a rising one under the lows, and a close below the rising one.
fn detect(pole: &Pole, _max_retrace: f64) -> f64 {
    let lines = &pole.lines;
    if pole.direction < 0
        && lines.upper.slope < 0.0
        && lines.lower.slope > 0.0
        && pole.close < lines.lower.level
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartBearPennant;

pub type ChartBearPennantStream = BarStream<ChartBearPennant, 3, 1>;

impl Kernel<3, 1> for ChartBearPennant {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_bear_pennant"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        whole(
            NAME,
            "pole_bars",
            params.pole_bars,
            POLE_BARS_MIN,
            POLE_BARS_MAX,
        )?;
        at_least(NAME, "min_pole", params.min_pole, MIN_POLE_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        PoleState::lookback(params.period, params.pole_bars)
    }

    fn state(params: &Params) -> State {
        PoleState::new(
            params.pivot_n,
            params.period,
            params.pole_bars,
            params.min_pole,
            f64::INFINITY,
            detect,
        )
    }
}
