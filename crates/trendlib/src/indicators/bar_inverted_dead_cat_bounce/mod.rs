use crate::core::bars::{BarHistory, BarState};
use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_inverted_dead_cat_bounce";

pub const RISE_PCT_DEFAULT: f64 = 0.1;
pub const RISE_PCT_MIN: f64 = 0.0;
pub const PULLBACK_PCT_DEFAULT: f64 = 0.5;
pub const PULLBACK_PCT_MIN: f64 = 0.0;
pub const PERIOD_DEFAULT: usize = 15;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub rise_pct: f64,
    pub pullback_pct: f64,
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            rise_pct: RISE_PCT_DEFAULT,
            pullback_pct: PULLBACK_PCT_DEFAULT,
            period: PERIOD_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// A rise of at least `rise_pct`, then a pullback of between 5 percent and
/// `pullback_pct` of it.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let half = params.period / 2;
    let start = history.back(half + params.period).close;
    let peak = (1..=half)
        .map(|back| history.back(back).close)
        .fold(f64::NEG_INFINITY, f64::max);
    if (peak - start) / (start + EPS) < params.rise_pct {
        return 0.0;
    }
    let pullback = (peak - history.back(0).close) / (peak - start + EPS);
    if 0.05 < pullback && pullback < params.pullback_pct {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarInvertedDeadCatBounce;

pub type BarInvertedDeadCatBounceStream = BarStream<BarInvertedDeadCatBounce, 4, 1>;

impl Kernel<4, 1> for BarInvertedDeadCatBounce {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_inverted_dead_cat_bounce"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "rise_pct", params.rise_pct, RISE_PCT_MIN)?;
        at_least(NAME, "pullback_pct", params.pullback_pct, PULLBACK_PCT_MIN)?;
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        2 * params.period
    }

    fn state(params: &Params) -> State {
        BarState::new(
            params.period + params.period / 2,
            2 * params.period,
            *params,
            rule,
        )
    }
}
