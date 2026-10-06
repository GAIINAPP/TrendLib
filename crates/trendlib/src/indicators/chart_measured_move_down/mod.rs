use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::MeasuredMoveState;

pub const NAME: &str = "chart_measured_move_down";

pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const LEG_TOL_DEFAULT: f64 = 0.2;
pub const LEG_TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub pivot_n: usize,
    pub leg_tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            pivot_n: PIVOT_N_DEFAULT,
            leg_tol: LEG_TOL_DEFAULT,
        }
    }
}

pub type State = MeasuredMoveState;

#[derive(Clone, Copy, Debug)]
pub struct ChartMeasuredMoveDown;

pub type ChartMeasuredMoveDownStream = BarStream<ChartMeasuredMoveDown, 3, 1>;

impl Kernel<3, 1> for ChartMeasuredMoveDown {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_measured_move_down"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "leg_tol", params.leg_tol, LEG_TOL_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        MeasuredMoveState::lookback(params.pivot_n)
    }

    fn state(params: &Params) -> State {
        MeasuredMoveState::new(true, params.pivot_n, params.leg_tol)
    }
}
