use crate::core::bars::BarHistory;
use crate::core::chart::EPS;
use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::HlcState;

pub const NAME: &str = "chart_high_tight_flag";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const POLE_BARS_DEFAULT: usize = 10;
pub const POLE_BARS_MIN: usize = 1;
pub const POLE_BARS_MAX: usize = 100_000;
pub const MIN_POLE_DEFAULT: f64 = 0.4;
pub const MIN_POLE_MIN: f64 = 0.0;
pub const MAX_RETRACE_DEFAULT: f64 = 0.2;
pub const MAX_RETRACE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pole_bars: usize,
    pub min_pole: f64,
    pub max_retrace: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pole_bars: POLE_BARS_DEFAULT,
            min_pole: MIN_POLE_DEFAULT,
            max_retrace: MAX_RETRACE_DEFAULT,
        }
    }
}

pub type State = HlcState<Params>;

/// A sharp rise, a tight range since, and a close above that range.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let pole_end = history.back(params.period).close;
    let pole_start = history.back(params.period + params.pole_bars).close;
    if (pole_end - pole_start) / (pole_start + EPS) < params.min_pole {
        return 0.0;
    }
    let flag = (1..=params.period).map(|back| history.back(back));
    let high = flag
        .clone()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = flag.map(|bar| bar.low).fold(f64::INFINITY, f64::min);
    if (high - low) / (pole_end + EPS) > params.max_retrace {
        return 0.0;
    }
    if history.back(0).close > high {
        100.0
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ChartHighTightFlag;

pub type ChartHighTightFlagStream = BarStream<ChartHighTightFlag, 3, 1>;

impl Kernel<3, 1> for ChartHighTightFlag {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_high_tight_flag"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(
            NAME,
            "pole_bars",
            params.pole_bars,
            POLE_BARS_MIN,
            POLE_BARS_MAX,
        )?;
        at_least(NAME, "min_pole", params.min_pole, MIN_POLE_MIN)?;
        at_least(NAME, "max_retrace", params.max_retrace, MAX_RETRACE_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period + params.pole_bars
    }

    fn state(params: &Params) -> State {
        HlcState::new(
            params.period + params.pole_bars,
            params.period + params.pole_bars,
            *params,
            rule,
        )
    }
}
