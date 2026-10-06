use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::BumpState;

pub const NAME: &str = "chart_bump_and_run_bottom";

pub const LEAD_WINDOW_DEFAULT: usize = 30;
pub const LEAD_WINDOW_MIN: usize = 2;
pub const LEAD_WINDOW_MAX: usize = 100_000;
pub const BUMP_WINDOW_DEFAULT: usize = 15;
pub const BUMP_WINDOW_MIN: usize = 2;
pub const BUMP_WINDOW_MAX: usize = 100_000;
pub const BUMP_FACTOR_DEFAULT: f64 = 2.0;
pub const BUMP_FACTOR_MIN: f64 = 0.0;
pub const PIVOT_N_DEFAULT: usize = 4;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub lead_window: usize,
    pub bump_window: usize,
    pub bump_factor: f64,
    pub pivot_n: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            lead_window: LEAD_WINDOW_DEFAULT,
            bump_window: BUMP_WINDOW_DEFAULT,
            bump_factor: BUMP_FACTOR_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
        }
    }
}

pub type State = BumpState;

#[derive(Clone, Copy, Debug)]
pub struct ChartBumpAndRunBottom;

pub type ChartBumpAndRunBottomStream = BarStream<ChartBumpAndRunBottom, 3, 1>;

impl Kernel<3, 1> for ChartBumpAndRunBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_bump_and_run_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "lead_window",
            params.lead_window,
            LEAD_WINDOW_MIN,
            LEAD_WINDOW_MAX,
        )?;
        whole(
            NAME,
            "bump_window",
            params.bump_window,
            BUMP_WINDOW_MIN,
            BUMP_WINDOW_MAX,
        )?;
        at_least(NAME, "bump_factor", params.bump_factor, BUMP_FACTOR_MIN)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        BumpState::lookback(params.lead_window, params.bump_window)
    }

    fn state(params: &Params) -> State {
        BumpState::new(
            true,
            params.lead_window,
            params.bump_window,
            params.bump_factor,
            params.pivot_n,
        )
    }
}
