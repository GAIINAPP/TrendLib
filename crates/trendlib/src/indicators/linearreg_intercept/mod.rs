use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Fit, LinearRegression};

pub const NAME: &str = "linearreg_intercept";

pub const PERIOD_DEFAULT: usize = 14;
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
    line: LinearRegression,
}

fn read(fit: Fit) -> [f64; 1] {
    [fit.intercept]
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.line.push(bar[0]).map(read)
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.line.preview(bar[0]).map(read)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LinearregIntercept;

pub type LinearregInterceptStream = BarStream<LinearregIntercept, 1, 1>;

impl Kernel<1, 1> for LinearregIntercept {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["linearreg_intercept"];

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
            line: LinearRegression::new(params.period),
        }
    }
}
