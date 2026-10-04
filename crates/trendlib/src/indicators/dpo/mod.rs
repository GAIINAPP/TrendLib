use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, RollingMean};

pub const NAME: &str = "dpo";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

/// How far back the value compared against the average is taken from.
fn shift(period: usize) -> usize {
    period / 2 + 1
}

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
    average: RollingMean,
    earlier: Lagged,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        // Both have to take the bar; the lag is longer than the average's
        // warm-up at small periods and shorter at large ones, so neither can
        // be the one that bails first.
        let average = self.average.push(bar[0]);
        let earlier = self.earlier.push(bar[0]);
        Some([earlier? - average?])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let average = self.average.preview(bar[0])?;
        let earlier = self.earlier.earlier()?;
        Some([earlier - average])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dpo;

pub type DpoStream = BarStream<Dpo, 1, 1>;

impl Kernel<1, 1> for Dpo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["dpo"];

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
        params.period.saturating_sub(1).max(shift(params.period))
    }

    fn state(params: &Params) -> State {
        State {
            average: RollingMean::new(params.period),
            earlier: Lagged::new(shift(params.period)),
        }
    }
}
