use crate::core::error::TlError;
use crate::core::hilbert::{Hilbert, PRIMED_EARLY};
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ht_phasor";

/// Taken from the oracle: twelve bars to start the transform and twenty more
/// for the cycle length to settle.
pub const LOOKBACK: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    cycle: Hilbert,
    seen: usize,
}

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let phase = self.cycle.push(bar[0]);
        self.seen += 1;
        let phase = phase?;
        (self.seen > LOOKBACK).then_some([phase.in_phase, phase.quadrature])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let phase = self.cycle.preview(bar[0])?;
        (self.seen + 1 > LOOKBACK).then_some([phase.in_phase, phase.quadrature])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HtPhasor;

pub type HtPhasorStream = BarStream<HtPhasor, 1, 2>;

impl Kernel<1, 2> for HtPhasor {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["ht_phasor_in_phase", "ht_phasor_quadrature"];

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
            cycle: Hilbert::new(PRIMED_EARLY),
            seen: 0,
        }
    }
}
