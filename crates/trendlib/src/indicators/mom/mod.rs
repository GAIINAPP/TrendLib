use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Lagged;

pub const NAME: &str = "mom";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 1;
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
pub struct State(Lagged);

fn combine(now: f64, earlier: f64) -> f64 {
    now - earlier
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let earlier = self.0.push(bar[0])?;
        Some([combine(bar[0], earlier)])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let earlier = self.0.earlier()?;
        Some([combine(bar[0], earlier)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mom;

pub type MomStream = BarStream<Mom, 1, 1>;

impl Kernel<1, 1> for Mom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["mom"];

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
        params.period
    }

    fn state(params: &Params) -> State {
        State(Lagged::new(params.period))
    }
}
