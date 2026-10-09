use crate::core::error::TlError;
use crate::core::hilbert::{Hilbert, PRIMED_LATE, Recent};
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ht_trendline";

/// Taken from the oracle: thirty-seven bars to start the transform and
/// twenty-six more for what it reads to settle.
pub const LOOKBACK: usize = 63;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    cycle: Hilbert,
    prices: Recent,
    /// The last three cycle averages, newest first.
    averaged: [f64; 3],
    seen: usize,
}

impl State {
    /// The weighted blend TA-Lib smooths the cycle average with, which it
    /// writes as fused multiply-adds.
    fn blend(&self, mean: f64) -> f64 {
        let inner = 4.0f64.mul_add(mean, 3.0 * self.averaged[0]);
        (2.0f64.mul_add(self.averaged[1], inner) + self.averaged[2]) / 10.0
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.prices.push(bar[0]);
        let phase = self.cycle.push(bar[0]);
        self.seen += 1;
        let phase = phase?;
        let mean = self.prices.mean((phase.smooth_period + 0.5) as usize);
        let value = self.blend(mean);
        self.averaged.rotate_right(1);
        self.averaged[0] = mean;
        (self.seen > LOOKBACK).then_some([value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let phase = self.cycle.preview(bar[0])?;
        let mut prices = self.prices;
        prices.push(bar[0]);
        let mean = prices.mean((phase.smooth_period + 0.5) as usize);
        (self.seen + 1 > LOOKBACK).then_some([self.blend(mean)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HtTrendline;

pub type HtTrendlineStream = BarStream<HtTrendline, 1, 1>;

impl Kernel<1, 1> for HtTrendline {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ht_trendline"];

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
            cycle: Hilbert::new(PRIMED_LATE),
            prices: Recent::default(),
            averaged: [0.0; 3],
            seen: 0,
        }
    }
}
