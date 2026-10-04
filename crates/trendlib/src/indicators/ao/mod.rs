use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingMean;

pub const NAME: &str = "ao";

pub const FAST_PERIOD_DEFAULT: usize = 5;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 34;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    fast: RollingMean,
    slow: RollingMean,
}

fn midpoint(bar: [f64; 2]) -> f64 {
    (bar[0] + bar[1]) / 2.0
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let value = midpoint(bar);
        // The shorter average is ready first and has to keep taking bars while
        // the longer one warms up.
        let fast = self.fast.push(value);
        let slow = self.slow.push(value)?;
        Some([fast? - slow])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let value = midpoint(bar);
        Some([self.fast.preview(value)? - self.slow.preview(value)?])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ao;

pub type AoStream = BarStream<Ao, 2, 1>;

impl Kernel<2, 1> for Ao {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["ao"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FAST_PERIOD_MIN..=FAST_PERIOD_MAX).contains(&params.fast_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "fast_period",
                params.fast_period,
                FAST_PERIOD_MIN,
                FAST_PERIOD_MAX,
            ));
        }
        if !(SLOW_PERIOD_MIN..=SLOW_PERIOD_MAX).contains(&params.slow_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slow_period",
                params.slow_period,
                SLOW_PERIOD_MIN,
                SLOW_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.fast_period.max(params.slow_period).saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            fast: RollingMean::new(params.fast_period),
            slow: RollingMean::new(params.slow_period),
        }
    }
}
