use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Ema, Lagged};

pub const NAME: &str = "trix";

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
    first: Ema,
    second: Ema,
    third: Ema,
    previous: Lagged,
}

/// The same reading as `rocp`: a base of exactly zero leaves the change
/// undefined, so the row is `NaN` rather than an infinity.
fn change(now: f64, earlier: f64) -> f64 {
    if earlier == 0.0 {
        f64::NAN
    } else {
        (now - earlier) / earlier * 100.0
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let first = self.first.push(bar[0])?;
        let second = self.second.push(first)?;
        let third = self.third.push(second)?;
        let earlier = self.previous.push(third)?;
        Some([change(third, earlier)])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let first = self.first.preview(bar[0])?;
        let second = self.second.preview(first)?;
        let third = self.third.preview(second)?;
        let earlier = self.previous.earlier()?;
        Some([change(third, earlier)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Trix;

pub type TrixStream = BarStream<Trix, 1, 1>;

impl Kernel<1, 1> for Trix {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["trix"];

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

    // Three exponential stages in series, then one bar to compare against.
    fn lookback(params: &Params) -> usize {
        3 * params.period.saturating_sub(1) + 1
    }

    fn state(params: &Params) -> State {
        State {
            first: Ema::new(params.period),
            second: Ema::new(params.period),
            third: Ema::new(params.period),
            previous: Lagged::new(1),
        }
    }
}
