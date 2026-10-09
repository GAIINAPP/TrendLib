use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_wide_ranging_day";

pub const FACTOR_DEFAULT: f64 = 2.0;
pub const FACTOR_MIN: f64 = 0.0;
pub const LOOKBACK_DEFAULT: usize = 10;
pub const LOOKBACK_MIN: usize = 1;
pub const LOOKBACK_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub factor: f64,
    pub lookback: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            factor: FACTOR_DEFAULT,
            lookback: LOOKBACK_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// A range of more than `factor` times the average of the bars before it,
/// closing in its upper or lower half.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let bar = history.back(0);
    let total: f64 = (1..=params.lookback)
        .map(|back| {
            let earlier = history.back(back);
            earlier.high - earlier.low
        })
        .sum();
    let average = total / params.lookback as f64;
    let wide = bar.high - bar.low > params.factor * average;
    let middle = (bar.high + bar.low) / 2.0;
    combine(wide && bar.close > middle, wide && bar.close < middle)
}

#[derive(Clone, Copy, Debug)]
pub struct BarWideRangingDay;

pub type BarWideRangingDayStream = BarStream<BarWideRangingDay, 4, 1>;

impl Kernel<4, 1> for BarWideRangingDay {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_wide_ranging_day"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "factor", params.factor, FACTOR_MIN)?;
        whole(
            NAME,
            "lookback",
            params.lookback,
            LOOKBACK_MIN,
            LOOKBACK_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.lookback
    }

    fn state(params: &Params) -> State {
        BarState::new(params.lookback, params.lookback, *params, rule)
    }
}
