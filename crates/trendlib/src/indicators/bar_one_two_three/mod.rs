use crate::core::bars::combine;
use crate::core::chart::whole;
use crate::core::chart::{Swing, SwingFinder, SwingLog};
use crate::core::error::TlError;
use crate::core::kernel::Step;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_one_two_three";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 3;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
        }
    }
}

/// Swing points on one side, each priced at the bar that confirmed it.
///
/// The oracle reads a swing's price at its confirmation bar, `pivot_n` bars
/// after the extreme, not at the extreme itself; this is that reading.
#[derive(Clone, Debug)]
pub struct State {
    highs: SwingFinder,
    lows: SwingFinder,
    peaks: SwingLog,
    troughs: SwingLog,
    period: usize,
    bars: usize,
}

impl State {
    /// Two swing points of one side in the window, the second less extreme,
    /// and a close through the opposite swing between them.
    fn half(main: &SwingLog, other: &SwingLog, oldest: usize, close: f64, bottom: bool) -> bool {
        let inside = main.iter().filter(|s| s.at >= oldest);
        let count = inside.clone().count();
        if count < 2 {
            return false;
        }
        let mut last_two = inside.skip(count - 2);
        let (Some(first), Some(third)) = (last_two.next(), last_two.next()) else {
            return false;
        };
        let Some(second) = other.iter().find(|s| s.at > first.at) else {
            return false;
        };
        second.at < third.at
            && if bottom {
                third.price > first.price && close > second.price
            } else {
                third.price < first.price && close < second.price
            }
    }

    fn advance(&mut self, bar: [f64; 4]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let (high, low, close) = (bar[1], bar[2], bar[3]);
        // Read before this bar's own swing points are filed: the oracle counts
        // only those confirmed by an earlier bar.
        let value = (now >= self.period).then(|| {
            let oldest = now - self.period;
            combine(
                Self::half(&self.troughs, &self.peaks, oldest, close, true),
                Self::half(&self.peaks, &self.troughs, oldest, close, false),
            )
        });
        if self.highs.push(high).is_some() {
            self.peaks.push(Swing {
                at: now,
                price: high,
            });
        }
        if self.lows.push(low).is_some() {
            self.troughs.push(Swing {
                at: now,
                price: low,
            });
        }
        self.peaks.trim(now);
        self.troughs.trim(now);
        value
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarOneTwoThree;

pub type BarOneTwoThreeStream = BarStream<BarOneTwoThree, 4, 1>;

impl Kernel<4, 1> for BarOneTwoThree {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_one_two_three"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            highs: SwingFinder::highs(params.pivot_n),
            lows: SwingFinder::lows(params.pivot_n),
            peaks: SwingLog::new(params.period),
            troughs: SwingLog::new(params.period),
            period: params.period,
            bars: 0,
        }
    }
}
