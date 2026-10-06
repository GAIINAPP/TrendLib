use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::DiamondState;

pub const NAME: &str = "chart_diamond_bottom";

pub const PERIOD_DEFAULT: usize = 60;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 4;
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

pub type State = DiamondState;

#[derive(Clone, Copy, Debug)]
pub struct ChartDiamondBottom;

pub type ChartDiamondBottomStream = BarStream<ChartDiamondBottom, 3, 1>;

impl Kernel<3, 1> for ChartDiamondBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_diamond_bottom"];

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
        DiamondState::new(true, params.period, params.pivot_n)
    }
}
