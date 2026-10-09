use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{TrueRange, Wilder};

pub const NAME: &str = "atr";

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
pub struct State {
    range: TrueRange,
    average: Wilder,
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let range = self.range.push(bar[0], bar[1], bar[2])?;
        self.average.push(range).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let range = self.range.preview(bar[0], bar[1])?;
        self.average.preview(range).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Atr;

pub type AtrStream = BarStream<Atr, 3, 1>;

impl Kernel<3, 1> for Atr {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["atr"];

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

    // One bar is consumed before the first true range exists, and the Wilder
    // average then needs `period` of them.
    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            range: TrueRange::new(),
            average: Wilder::new(params.period),
        }
    }
}
