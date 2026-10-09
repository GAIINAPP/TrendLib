use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::{percentrank, rsi};

pub const NAME: &str = "connors_rsi";

pub const RSI_PERIOD_DEFAULT: usize = 3;
pub const RSI_PERIOD_MIN: usize = 2;
pub const RSI_PERIOD_MAX: usize = 100_000;
pub const STREAK_PERIOD_DEFAULT: usize = 2;
pub const STREAK_PERIOD_MIN: usize = 2;
pub const STREAK_PERIOD_MAX: usize = 100_000;
pub const RANK_PERIOD_DEFAULT: usize = 100;
pub const RANK_PERIOD_MIN: usize = 1;
pub const RANK_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub rsi_period: usize,
    pub streak_period: usize,
    pub rank_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            rsi_period: RSI_PERIOD_DEFAULT,
            streak_period: STREAK_PERIOD_DEFAULT,
            rank_period: RANK_PERIOD_DEFAULT,
        }
    }
}

// TA-Lib's arithmetic for a one-bar rate of change, ratio first. `roc` divides
// the difference instead, which rounds differently by up to 1e-10.
fn change(close: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        0.0
    } else {
        ((close / previous) - 1.0) * 100.0
    }
}

#[derive(Clone, Debug)]
pub struct State {
    price: rsi::State,
    streak_rsi: rsi::State,
    rank: percentrank::State,
    previous: Option<f64>,
    streak: f64,
}

impl State {
    fn advance(&mut self, close: f64) -> Option<f64> {
        let price = self.price.push([close]);
        let (ranked, streaked) = match self.previous {
            Some(previous) => {
                self.streak = if close > previous {
                    self.streak.max(0.0) + 1.0
                } else if close < previous {
                    self.streak.min(0.0) - 1.0
                } else {
                    0.0
                };
                (
                    self.rank.push([change(close, previous)]),
                    self.streak_rsi.push([self.streak]),
                )
            }
            None => (None, None),
        };
        self.previous = Some(close);
        let ([price], [streaked], [ranked]) = (price?, streaked?, ranked?);
        Some((price + streaked + ranked) / 3.0)
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
pub struct ConnorsRsi;

pub type ConnorsRsiStream = BarStream<ConnorsRsi, 1, 1>;

impl Kernel<1, 1> for ConnorsRsi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["connors_rsi"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "rsi_period",
            params.rsi_period,
            RSI_PERIOD_MIN,
            RSI_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "streak_period",
            params.streak_period,
            STREAK_PERIOD_MIN,
            STREAK_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "rank_period",
            params.rank_period,
            RANK_PERIOD_MIN,
            RANK_PERIOD_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params
            .rsi_period
            .max(1 + params.streak_period)
            .max(1 + params.rank_period)
    }

    fn state(params: &Params) -> State {
        State {
            price: rsi::Rsi::state(&rsi::Params {
                period: params.rsi_period,
            }),
            streak_rsi: rsi::Rsi::state(&rsi::Params {
                period: params.streak_period,
            }),
            rank: percentrank::Percentrank::state(&percentrank::Params {
                period: params.rank_period,
            }),
            previous: None,
            streak: 0.0,
        }
    }
}
