use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Paired, Sums};

pub const NAME: &str = "beta";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 1;
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
    previous: Option<(f64, f64)>,
    returns: Paired,
}

/// A window in which the second series never moved has no slope to fit, and
/// TA-Lib answers zero rather than dividing. The test is exact.
fn slope(sums: Sums) -> f64 {
    if sums.first_spread == 0.0 {
        0.0
    } else {
        sums.shared / sums.first_spread
    }
}

fn returns(now: (f64, f64), previous: (f64, f64)) -> (f64, f64) {
    (
        (now.0 - previous.0) / previous.0,
        (now.1 - previous.1) / previous.1,
    )
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let now = (bar[0], bar[1]);
        let previous = self.previous.replace(now)?;
        let (first, second) = returns(now, previous);
        self.returns.push(first, second).map(|sums| [slope(sums)])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let (first, second) = returns((bar[0], bar[1]), self.previous?);
        self.returns
            .preview(first, second)
            .map(|sums| [slope(sums)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Beta;

pub type BetaStream = BarStream<Beta, 2, 1>;

impl Kernel<2, 1> for Beta {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["source0", "source1"];
    const OUTPUTS: [&'static str; 1] = ["beta"];

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
            previous: None,
            returns: Paired::new(params.period),
        }
    }
}
