use crate::core::chart::RepeatState;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_big_m";

pub const PERIOD_DEFAULT: usize = 150;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 7;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const TOL_DEFAULT: f64 = 0.03;
pub const TOL_MIN: f64 = 0.0;
pub const MIN_SEP_DEFAULT: usize = 15;
pub const MIN_SEP_MIN: usize = 1;
pub const MIN_SEP_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub tol: f64,
    pub min_sep: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            tol: TOL_DEFAULT,
            min_sep: MIN_SEP_DEFAULT,
        }
    }
}

pub type State = RepeatState;

#[derive(Clone, Copy, Debug)]
pub struct ChartBigM;

pub type ChartBigMStream = BarStream<ChartBigM, 3, 1>;

impl Kernel<3, 1> for ChartBigM {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_big_m"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "tol", params.tol, TOL_MIN)?;
        whole(NAME, "min_sep", params.min_sep, MIN_SEP_MIN, MIN_SEP_MAX)?;
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
            params.min_sep,
        )
    }
}
