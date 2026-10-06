use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::StairState;

pub const NAME: &str = "chart_three_peaks";

pub const PERIOD_DEFAULT: usize = 120;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const MIN_SEPARATION_DEFAULT: usize = 8;
pub const MIN_SEPARATION_MIN: usize = 1;
pub const MIN_SEPARATION_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub min_separation: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            min_separation: MIN_SEPARATION_DEFAULT,
        }
    }
}

pub type State = StairState;

#[derive(Clone, Copy, Debug)]
pub struct ChartThreePeaks;

pub type ChartThreePeaksStream = BarStream<ChartThreePeaks, 3, 1>;

impl Kernel<3, 1> for ChartThreePeaks {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_three_peaks"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
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
        StairState::new(false, params.period, params.pivot_n, params.min_separation)
    }
}
