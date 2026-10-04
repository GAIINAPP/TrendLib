use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Ema, Lagged};

pub const NAME: &str = "cvi";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100000;
pub const ROC_PERIOD_DEFAULT: usize = 10;
pub const ROC_PERIOD_MIN: usize = 1;
pub const ROC_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
    pub roc_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            roc_period: ROC_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    smoothed: Ema,
    earlier: Lagged,
}

/// A series that never moves smooths to a range of zero, and TA-Lib answers a
/// change of zero rather than dividing. The test is exact.
fn change(now: f64, earlier: f64) -> f64 {
    if earlier == 0.0 {
        0.0
    } else {
        (now - earlier) / earlier * 100.0
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let now = self.smoothed.push(bar[0] - bar[1])?;
        let earlier = self.earlier.push(now)?;
        Some([change(now, earlier)])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let now = self.smoothed.preview(bar[0] - bar[1])?;
        let earlier = self.earlier.earlier()?;
        Some([change(now, earlier)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cvi;

pub type CviStream = BarStream<Cvi, 2, 1>;

impl Kernel<2, 1> for Cvi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["cvi"];

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
        if !(ROC_PERIOD_MIN..=ROC_PERIOD_MAX).contains(&params.roc_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "roc_period",
                params.roc_period,
                ROC_PERIOD_MIN,
                ROC_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1) + params.roc_period
    }

    fn state(params: &Params) -> State {
        State {
            smoothed: Ema::new(params.period),
            earlier: Lagged::new(params.roc_period),
        }
    }
}
