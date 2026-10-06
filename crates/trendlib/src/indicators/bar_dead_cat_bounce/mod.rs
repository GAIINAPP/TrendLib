use crate::core::bars::{BarHistory, BarState};
use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_dead_cat_bounce";

pub const DROP_PCT_DEFAULT: f64 = 0.1;
pub const DROP_PCT_MIN: f64 = 0.0;
pub const BOUNCE_PCT_DEFAULT: f64 = 0.5;
pub const BOUNCE_PCT_MIN: f64 = 0.0;
pub const PERIOD_DEFAULT: usize = 15;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub drop_pct: f64,
    pub bounce_pct: f64,
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            drop_pct: DROP_PCT_DEFAULT,
            bounce_pct: BOUNCE_PCT_DEFAULT,
            period: PERIOD_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// A fall of at least `drop_pct`, then a bounce of between 5 percent and
/// `bounce_pct` of it.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let half = params.period / 2;
    let start = history.back(half + params.period).close;
    let trough = (1..=half)
        .map(|back| history.back(back).close)
        .fold(f64::INFINITY, f64::min);
    if (start - trough) / (start + EPS) < params.drop_pct {
        return 0.0;
    }
    let bounce = (history.back(0).close - trough) / (start - trough + EPS);
    if 0.05 < bounce && bounce < params.bounce_pct {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarDeadCatBounce;

pub type BarDeadCatBounceStream = BarStream<BarDeadCatBounce, 4, 1>;

impl Kernel<4, 1> for BarDeadCatBounce {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_dead_cat_bounce"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "drop_pct", params.drop_pct, DROP_PCT_MIN)?;
        at_least(NAME, "bounce_pct", params.bounce_pct, BOUNCE_PCT_MIN)?;
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
