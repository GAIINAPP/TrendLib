use crate::core::candles::{Candle, CandleHistory, CandleSetting, Pending};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "cdl_hikkakemod";

/// Taken from the oracle: five bars of pattern over the five-bar `near`
/// average that qualifies the first of them.
pub const LOOKBACK: usize = 10;

/// Where the state machine starts running, three bars before the first row it
/// reports.
const PRIMED_FROM: usize = LOOKBACK - 3;

/// Bars the history has to hold: the four the pattern reads, plus the five the
/// `near` average covers behind the second of them.
const DEPTH: usize = 7;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    history: CandleHistory,
    seen: usize,
    pending: Option<Pending>,
}

impl State {
    /// The setup: two inside bars in a row, a break out of the second, and a
    /// first bar that closed at the end of its own range in the direction the
    /// break is read as.
    fn broken_out(&self) -> Option<f64> {
        let bar = self.history.back(0);
        let inside = self.history.back(1);
        let before = self.history.back(2);
        let first = self.history.back(3);
        let nested = before.high < first.high
            && before.low > first.low
            && inside.high < before.high
            && inside.low > before.low;
        if !nested {
            return None;
        }
        let near = self.history.average(CandleSetting::NEAR, 2);
        if bar.high < inside.high && bar.low < inside.low && before.close <= before.low + near {
            Some(100.0)
        } else if bar.high > inside.high
            && bar.low > inside.low
            && before.close >= before.high - near
        {
            Some(-100.0)
        } else {
            None
        }
    }

    fn step(&mut self) -> f64 {
        let bar = self.history.back(0);
        let inside = self.history.back(1);
        if let Some(result) = self.broken_out() {
            self.pending = Some(Pending {
                result,
                age: 0,
                high: inside.high,
                low: inside.low,
            });
            return result;
        }
        match self.pending {
            Some(pending) if pending.confirmed_by(bar.close) => {
                self.pending = None;
                pending.doubled()
            }
            _ => 0.0,
        }
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.history.push(Candle::new(bar));
        if let Some(pending) = &mut self.pending {
            pending.age += 1;
        }
        self.seen += 1;
        if self.seen <= PRIMED_FROM {
            return None;
        }
        let value = self.step();
        (self.seen > LOOKBACK).then_some([value])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlHikkakemod;

pub type CdlHikkakemodStream = BarStream<CdlHikkakemod, 4, 1>;

impl Kernel<4, 1> for CdlHikkakemod {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_hikkakemod"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(_params: &Params) -> State {
        State {
            history: CandleHistory::new(DEPTH),
            seen: 0,
            pending: None,
        }
    }
}
