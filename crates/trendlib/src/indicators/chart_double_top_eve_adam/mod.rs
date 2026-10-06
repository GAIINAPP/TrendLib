use crate::core::chart::RepeatState;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_double_top_eve_adam";

pub const PERIOD_DEFAULT: usize = 60;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 3;
pub const PIVOT_N_MAX: usize = 100_000;
pub const TOL_DEFAULT: f64 = 0.03;
pub const TOL_MIN: f64 = 0.0;
pub const MIN_SEPARATION_DEFAULT: usize = 5;
pub const MIN_SEPARATION_MIN: usize = 1;
pub const MIN_SEPARATION_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub tol: f64,
    pub min_separation: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            tol: TOL_DEFAULT,
            min_separation: MIN_SEPARATION_DEFAULT,
        }
    }
}

pub type State = RepeatState;

#[derive(Clone, Copy, Debug)]
pub struct ChartDoubleTopEveAdam;

pub type ChartDoubleTopEveAdamStream = BarStream<ChartDoubleTopEveAdam, 3, 1>;

impl Kernel<3, 1> for ChartDoubleTopEveAdam {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_double_top_eve_adam"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "tol", params.tol, TOL_MIN)?;
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
        params.period
    }

    fn state(params: &Params) -> State {
        RepeatState::new(
            2,
            true,
            params.pivot_n,
            params.period,
            params.tol,
            params.min_separation,
        )
        .with_shapes(false, true, params.pivot_n)
    }
}
