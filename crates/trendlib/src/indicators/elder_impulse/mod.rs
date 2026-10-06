use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;
use crate::indicators::macd;

pub const NAME: &str = "elder_impulse";

pub const EMA_PERIOD_DEFAULT: usize = 13;
pub const EMA_PERIOD_MIN: usize = 2;
pub const EMA_PERIOD_MAX: usize = 100_000;
pub const FAST_PERIOD_DEFAULT: usize = 12;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100_000;
pub const SLOW_PERIOD_DEFAULT: usize = 26;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100_000;
pub const SIGNAL_PERIOD_DEFAULT: usize = 9;
pub const SIGNAL_PERIOD_MIN: usize = 1;
pub const SIGNAL_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub ema_period: usize,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            ema_period: EMA_PERIOD_DEFAULT,
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

fn macd_params(params: &Params) -> macd::Params {
    macd::Params {
        fast_period: params.fast_period,
        slow_period: params.slow_period,
        signal_period: params.signal_period,
    }
}

fn impulse(trend: f64, was_trend: f64, histogram: f64, was_histogram: f64) -> f64 {
    if trend > was_trend && histogram > was_histogram {
        1.0
    } else if trend < was_trend && histogram < was_histogram {
        -1.0
    } else {
        0.0
    }
}

#[derive(Clone, Debug)]
pub struct State {
    trend: Ema,
    macd: macd::State,
    last: Option<(f64, f64)>,
}

impl State {
    fn advance(&mut self, close: f64) -> Option<f64> {
        let trend = self.trend.push(close);
        let histogram = self.macd.push([close]).map(|[_, _, histogram]| histogram);
        let (trend, histogram) = (trend?, histogram?);
        let reading = self
            .last
            .map(|(was_trend, was_histogram)| impulse(trend, was_trend, histogram, was_histogram));
        self.last = Some((trend, histogram));
        reading
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.advance(bar[0]).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.clone().advance(bar[0]).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ElderImpulse;

pub type ElderImpulseStream = BarStream<ElderImpulse, 1, 1>;

impl Kernel<1, 1> for ElderImpulse {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["elder_impulse"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "ema_period",
            params.ema_period,
            EMA_PERIOD_MIN,
            EMA_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "fast_period",
            params.fast_period,
            FAST_PERIOD_MIN,
            FAST_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "slow_period",
            params.slow_period,
            SLOW_PERIOD_MIN,
            SLOW_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "signal_period",
            params.signal_period,
            SIGNAL_PERIOD_MIN,
            SIGNAL_PERIOD_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        (params.ema_period - 1).max(macd::Macd::lookback(&macd_params(params))) + 1
    }

    fn state(params: &Params) -> State {
        State {
            trend: Ema::new(params.ema_period),
            macd: macd::Macd::state(&macd_params(params)),
            last: None,
        }
    }
}
