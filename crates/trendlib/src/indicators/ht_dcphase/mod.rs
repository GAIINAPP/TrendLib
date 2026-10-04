use crate::core::error::TlError;
use crate::core::hilbert::{DominantPhase, Hilbert, PRIMED_LATE};
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ht_dcphase";

/// Taken from the oracle: thirty-seven bars to start the transform and
/// twenty-six more for what it reads to settle.
pub const LOOKBACK: usize = 63;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    cycle: Hilbert,
    phase: DominantPhase,
    seen: usize,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let reading = self.cycle.push(bar[0]);
        self.seen += 1;
        let reading = reading?;
        let degrees = self.phase.push(&self.cycle, reading.smooth_period);
        (self.seen > LOOKBACK).then_some([degrees])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HtDcphase;

pub type HtDcphaseStream = BarStream<HtDcphase, 1, 1>;

impl Kernel<1, 1> for HtDcphase {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ht_dcphase"];

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
            phase: DominantPhase::default(),
            seen: 0,
        }
    }
}
