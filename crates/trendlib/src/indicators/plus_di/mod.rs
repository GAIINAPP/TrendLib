use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Directional;

pub const NAME: &str = "plus_di";

pub const PERIOD_DEFAULT: usize = 14;
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
pub struct State(Directional);

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.0.push(bar[0], bar[1], bar[2]).map(|pair| [pair.0])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.0.preview(bar[0], bar[1], bar[2]).map(|pair| [pair.0])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlusDi;

pub type PlusDiStream = BarStream<PlusDi, 3, 1>;

impl Kernel<3, 1> for PlusDi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["plus_di"];

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
        State(Directional::new(params.period))
    }
}
