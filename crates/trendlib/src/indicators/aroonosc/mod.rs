use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "aroonosc";

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
    highest: RollingExtreme,
    lowest: RollingExtreme,
    span: f64,
}

/// How recently the extreme was set, as a percentage of the window: 100 when it
/// is this bar, 0 when it is the oldest bar the window still holds.
fn freshness(bars_since: u64, span: f64) -> f64 {
    100.0 * (span - bars_since as f64) / span
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.highest.push(bar[0]);
        self.lowest.push(bar[1]);
        let high = self.highest.bars_since_best()?;
        let low = self.lowest.bars_since_best()?;
        Some([freshness(high, self.span) - freshness(low, self.span)])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let high = self.highest.preview_bars_since_best(bar[0])?;
        let low = self.lowest.preview_bars_since_best(bar[1])?;
        Some([freshness(high, self.span) - freshness(low, self.span)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Aroonosc;

pub type AroonoscStream = BarStream<Aroonosc, 2, 1>;

impl Kernel<2, 1> for Aroonosc {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["aroonosc"];

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
        // The window spans period + 1 bars: the current one and the period
        // bars before it, so an extreme set this bar reads 100.
        State {
            highest: RollingExtreme::highest(params.period + 1).preferring_recent(),
            lowest: RollingExtreme::lowest(params.period + 1).preferring_recent(),
            span: params.period as f64,
        }
    }
}
