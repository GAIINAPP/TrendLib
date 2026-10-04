use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Mesa;

pub const NAME: &str = "mama";

/// Taken from the oracle.
pub const LOOKBACK: usize = 32;

pub const FAST_LIMIT_DEFAULT: f64 = 0.5;
pub const FAST_LIMIT_MIN: f64 = 0.01;
pub const FAST_LIMIT_MAX: f64 = 0.99;
pub const SLOW_LIMIT_DEFAULT: f64 = 0.05;
pub const SLOW_LIMIT_MIN: f64 = 0.01;
pub const SLOW_LIMIT_MAX: f64 = 0.99;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub fast_limit: f64,
    pub slow_limit: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_limit: FAST_LIMIT_DEFAULT,
            slow_limit: SLOW_LIMIT_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State(Mesa);

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        self.0.push(bar[0]).map(|(mama, fama)| [mama, fama])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        self.0.preview(bar[0]).map(|(mama, fama)| [mama, fama])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mama;

pub type MamaStream = BarStream<Mama, 1, 2>;

impl Kernel<1, 2> for Mama {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["mama", "mama_fama"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FAST_LIMIT_MIN..=FAST_LIMIT_MAX).contains(&params.fast_limit) {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "fast_limit",
                params.fast_limit,
                FAST_LIMIT_MIN,
                FAST_LIMIT_MAX,
            ));
        }
        if !(SLOW_LIMIT_MIN..=SLOW_LIMIT_MAX).contains(&params.slow_limit) {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "slow_limit",
                params.slow_limit,
                SLOW_LIMIT_MIN,
                SLOW_LIMIT_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        State(Mesa::new(params.fast_limit, params.slow_limit))
    }
}
