use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingMean;

pub const NAME: &str = "trima";

pub const PERIOD_DEFAULT: usize = 30;
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
    first: RollingMean,
    second: RollingMean,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let first = self.first.push(bar[0])?;
        self.second.push(first).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let first = self.first.preview(bar[0])?;
        self.second.preview(first).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Trima;

pub type TrimaStream = BarStream<Trima, 1, 1>;

impl Kernel<1, 1> for Trima {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["trima"];

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
        // Averaging twice over halves of the window is what puts the triangular
        // weighting in: the middle bars are counted by both stages. An odd
        // period splits evenly; an even one takes the extra bar in the second
        // stage, which is where TA-Lib puts it.
        let half = params.period / 2;
        let (first, second) = if params.period % 2 == 1 {
            (half + 1, half + 1)
        } else {
            (half, half + 1)
        };
        State {
            first: RollingMean::new(first),
            second: RollingMean::new(second),
        }
    }
}
