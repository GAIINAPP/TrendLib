use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Tillson;

pub const NAME: &str = "t3";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;
pub const V_FACTOR_DEFAULT: f64 = Tillson::V_FACTOR_DEFAULT;
pub const V_FACTOR_MIN: f64 = 0.0;
pub const V_FACTOR_MAX: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub v_factor: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            v_factor: V_FACTOR_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State(Tillson);

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.0.push(bar[0]).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.0.preview(bar[0]).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct T3;

pub type T3Stream = BarStream<T3, 1, 1>;

impl Kernel<1, 1> for T3 {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["t3"];

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
        if !(V_FACTOR_MIN..=V_FACTOR_MAX).contains(&params.v_factor) {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "v_factor",
                params.v_factor,
                V_FACTOR_MIN,
                V_FACTOR_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        Tillson::lookback(params.period)
    }

    fn state(params: &Params) -> State {
        State(Tillson::new(params.period, params.v_factor))
    }
}
