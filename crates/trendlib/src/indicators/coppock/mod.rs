use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, WeightedMean};

pub const NAME: &str = "coppock";

pub const WMA_PERIOD_DEFAULT: usize = 10;
pub const WMA_PERIOD_MIN: usize = 1;
pub const WMA_PERIOD_MAX: usize = 100000;
pub const ROC1_PERIOD_DEFAULT: usize = 11;
pub const ROC1_PERIOD_MIN: usize = 1;
pub const ROC1_PERIOD_MAX: usize = 100000;
pub const ROC2_PERIOD_DEFAULT: usize = 14;
pub const ROC2_PERIOD_MIN: usize = 1;
pub const ROC2_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub wma_period: usize,
    pub roc1_period: usize,
    pub roc2_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            wma_period: WMA_PERIOD_DEFAULT,
            roc1_period: ROC1_PERIOD_DEFAULT,
            roc2_period: ROC2_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    first: Lagged,
    second: Lagged,
    smoothed: WeightedMean,
}

/// The percentage change against a value `n` bars back, as `roc` reads it.
fn change(now: f64, earlier: f64) -> f64 {
    (now - earlier) / earlier * 100.0
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        // Both lags have to take the bar before either can bail.
        let first = self.first.push(bar[0]);
        let second = self.second.push(bar[0])?;
        let total = change(bar[0], first?) + change(bar[0], second);
        self.smoothed.push(total).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let total = change(bar[0], self.first.earlier()?) + change(bar[0], self.second.earlier()?);
        self.smoothed.preview(total).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coppock;

pub type CoppockStream = BarStream<Coppock, 1, 1>;

impl Kernel<1, 1> for Coppock {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["coppock"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(WMA_PERIOD_MIN..=WMA_PERIOD_MAX).contains(&params.wma_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "wma_period",
                params.wma_period,
                WMA_PERIOD_MIN,
                WMA_PERIOD_MAX,
            ));
        }
        if !(ROC1_PERIOD_MIN..=ROC1_PERIOD_MAX).contains(&params.roc1_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "roc1_period",
                params.roc1_period,
                ROC1_PERIOD_MIN,
                ROC1_PERIOD_MAX,
            ));
        }
        if !(ROC2_PERIOD_MIN..=ROC2_PERIOD_MAX).contains(&params.roc2_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "roc2_period",
                params.roc2_period,
                ROC2_PERIOD_MIN,
                ROC2_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.roc1_period.max(params.roc2_period) + params.wma_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            first: Lagged::new(params.roc1_period),
            second: Lagged::new(params.roc2_period),
            smoothed: WeightedMean::new(params.wma_period),
        }
    }
}
