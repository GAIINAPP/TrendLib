use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::atr;

pub const NAME: &str = "wilder_volatility";

pub const PERIOD_DEFAULT: usize = 7;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;
pub const MULTIPLIER_DEFAULT: f64 = 3.0;
pub const MULTIPLIER_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            multiplier: MULTIPLIER_DEFAULT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Swing {
    direction: f64,
    extreme: f64,
    level: f64,
}

#[derive(Clone, Debug)]
pub struct State {
    range: atr::State,
    first_close: Option<f64>,
    multiplier: f64,
    swing: Option<Swing>,
}

impl State {
    fn advance(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let close = bar[2];
        let first = *self.first_close.get_or_insert(close);
        let [average] = self.range.push(bar)?;
        let (direction, extreme) = match self.swing {
            None if close >= first => (1.0, close),
            None => (-1.0, close),
            Some(swing) if swing.direction > 0.0 && close < swing.level => (-1.0, close),
            Some(swing) if swing.direction < 0.0 && close > swing.level => (1.0, close),
            Some(swing) if swing.direction > 0.0 => (1.0, swing.extreme.max(close)),
            Some(swing) => (-1.0, swing.extreme.min(close)),
        };
        let reach = average * self.multiplier;
        let level = if direction > 0.0 {
            extreme - reach
        } else {
            extreme + reach
        };
        self.swing = Some(Swing {
            direction,
            extreme,
            level,
        });
        Some([level, direction])
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WilderVolatility;

pub type WilderVolatilityStream = BarStream<WilderVolatility, 3, 2>;

impl Kernel<3, 2> for WilderVolatility {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["wilder_volatility", "wilder_volatility_direction"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "multiplier", params.multiplier, MULTIPLIER_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            range: atr::Atr::state(&atr::Params {
                period: params.period,
            }),
            first_close: None,
            multiplier: params.multiplier,
            swing: None,
        }
    }
}
