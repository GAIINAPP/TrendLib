use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Directional, Wilder, directional_index};

pub const NAME: &str = "adx";

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
    directional: Directional,
    average: Wilder,
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let (plus, minus) = self.directional.push(bar[0], bar[1], bar[2])?;
        self.average
            .push(directional_index(plus, minus))
            .map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let (plus, minus) = self.directional.preview(bar[0], bar[1], bar[2])?;
        self.average
            .preview(directional_index(plus, minus))
            .map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Adx;

pub type AdxStream = BarStream<Adx, 3, 1>;

impl Kernel<3, 1> for Adx {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["adx"];

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
        2 * params.period - 1
    }

    fn state(params: &Params) -> State {
        State {
            directional: Directional::new(params.period),
            average: Wilder::new(params.period),
        }
    }
}
