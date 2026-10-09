use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Ema, RollingWindow};

pub const NAME: &str = "massi";

pub const FAST_PERIOD_DEFAULT: usize = 9;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 25;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    first: Ema,
    second: Ema,
    ratios: RollingWindow,
}

/// A series that never moves smooths to a range of zero on both stages, and
/// TA-Lib reads the ratio as 1 rather than dividing. The test is exact.
fn ratio(once: f64, twice: f64) -> f64 {
    if twice == 0.0 { 1.0 } else { once / twice }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let once = self.first.push(bar[0] - bar[1])?;
        let twice = self.second.push(once)?;
        self.ratios
            .push(ratio(once, twice))
            .then(|| [self.ratios.sum()])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let once = self.first.preview(bar[0] - bar[1])?;
        let twice = self.second.preview(once)?;
        self.ratios
            .preview_is_full()
            .then(|| [self.ratios.preview_sum(ratio(once, twice))])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Massi;

pub type MassiStream = BarStream<Massi, 2, 1>;

impl Kernel<2, 1> for Massi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["massi"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FAST_PERIOD_MIN..=FAST_PERIOD_MAX).contains(&params.fast_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "fast_period",
                params.fast_period,
                FAST_PERIOD_MIN,
                FAST_PERIOD_MAX,
            ));
        }
        if !(SLOW_PERIOD_MIN..=SLOW_PERIOD_MAX).contains(&params.slow_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slow_period",
                params.slow_period,
                SLOW_PERIOD_MIN,
                SLOW_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        2 * params.fast_period.saturating_sub(1) + params.slow_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            first: Ema::new(params.fast_period),
            second: Ema::new(params.fast_period),
            ratios: RollingWindow::new(params.slow_period),
        }
    }
}
