use crate::core::chart::{ShouldersState, at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_head_shoulders";

/// Shoulders and head are swing highs, the neckline runs through swing lows.
const TOP: bool = true;

pub const PERIOD_DEFAULT: usize = 150;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const SHOULDER_TOL_DEFAULT: f64 = 0.05;
pub const SHOULDER_TOL_MIN: f64 = 0.0;
pub const MIN_SEPARATION_DEFAULT: usize = 8;
pub const MIN_SEPARATION_MIN: usize = 1;
pub const MIN_SEPARATION_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub shoulder_tol: f64,
    pub min_separation: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            shoulder_tol: SHOULDER_TOL_DEFAULT,
            min_separation: MIN_SEPARATION_DEFAULT,
        }
    }
}

pub type State = ShouldersState;

#[derive(Clone, Copy, Debug)]
pub struct ChartHeadShoulders;

pub type ChartHeadShouldersStream = BarStream<ChartHeadShoulders, 3, 1>;

impl Kernel<3, 1> for ChartHeadShoulders {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_head_shoulders"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "shoulder_tol", params.shoulder_tol, SHOULDER_TOL_MIN)?;
        whole(
            NAME,
            "min_separation",
            params.min_separation,
            MIN_SEPARATION_MIN,
            MIN_SEPARATION_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        ShouldersState::lookback(params.pivot_n, params.min_separation)
    }

    fn state(params: &Params) -> State {
        ShouldersState::new(
            TOP,
            params.pivot_n,
            params.period,
            params.shoulder_tol,
            params.min_separation,
        )
    }
}
