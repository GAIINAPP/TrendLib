use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "minmax";

pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    lowest: RollingExtreme,
    highest: RollingExtreme,
}

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let lowest = self.lowest.push(bar[0]);
        let highest = self.highest.push(bar[0]);
        Some([lowest?, highest?])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let lowest = self.lowest.preview(bar[0])?;
        let highest = self.highest.preview(bar[0])?;
        Some([lowest, highest])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Minmax;

pub type MinmaxStream = BarStream<Minmax, 1, 2>;

impl Kernel<1, 2> for Minmax {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["minmax_min", "minmax_max"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            lowest: RollingExtreme::lowest(params.period),
            highest: RollingExtreme::highest(params.period),
        }
    }
}
