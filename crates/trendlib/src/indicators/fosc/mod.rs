use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, LinearRegression};

pub const NAME: &str = "fosc";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100000;

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
    forecast: Lagged,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        // The forecast for this bar was made on the last one, so the fit has
        // to be taken first and then held back a bar.
        let made = self.line.push(bar[0]).map(|fit| fit.one_bar_ahead());
        let earlier = made.and_then(|value| self.forecast.push(value))?;
        Some([100.0 * (bar[0] - earlier) / bar[0]])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let earlier = self.forecast.earlier()?;
        Some([100.0 * (bar[0] - earlier) / bar[0]])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fosc;

pub type FoscStream = BarStream<Fosc, 1, 1>;

impl Kernel<1, 1> for Fosc {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["fosc"];

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
        State {
            line: LinearRegression::new(params.period),
            forecast: Lagged::new(1),
        }
    }
}
