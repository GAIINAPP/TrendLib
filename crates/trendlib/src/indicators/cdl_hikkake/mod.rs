use crate::core::candles::{Candle, CandleHistory, Pending};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "cdl_hikkake";

/// Taken from the oracle. The setup reads three bars and the state it leaves
/// is primed over the three before the first reported row.
pub const LOOKBACK: usize = 5;

/// Where the state machine starts running, three bars before the first row it
/// reports. A stream that began later would carry a different setup into its
/// first row, which is why this is `path_dependent`.
const PRIMED_FROM: usize = LOOKBACK - 3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    history: CandleHistory,
    seen: usize,
    pending: Option<Pending>,
}

/// The setup: a bar inside the one before it, then a bar that steps wholly
/// above or wholly below the inside bar.
fn broken_out(bar: Candle, inside: Candle, before: Candle) -> Option<f64> {
    if !(inside.high < before.high && inside.low > before.low) {
        return None;
    }
    if bar.high < inside.high && bar.low < inside.low {
        Some(100.0)
    } else if bar.high > inside.high && bar.low > inside.low {
        Some(-100.0)
    } else {
        None
    }
}

impl State {
    /// One bar of the state machine, returning what the row reads.
    fn step(&mut self) -> f64 {
        let bar = self.history.back(0);
        let inside = self.history.back(1);
        let before = self.history.back(2);
        if let Some(result) = broken_out(bar, inside, before) {
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
pub struct CdlHikkake;

pub type CdlHikkakeStream = BarStream<CdlHikkake, 4, 1>;

impl Kernel<4, 1> for CdlHikkake {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_hikkake"];

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
            history: CandleHistory::new(2),
            seen: 0,
            pending: None,
        }
    }
}
