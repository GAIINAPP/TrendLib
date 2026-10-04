use crate::core::error::TlError;
use crate::core::hilbert::{DominantPhase, Hilbert, PRIMED_LATE};
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ht_sine";

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

/// Degrees to radians, which is how far the two lines are apart as well: the
/// lead line is an eighth of a turn ahead.
const DEGREE: f64 = std::f64::consts::PI / 180.0;
const LEAD: f64 = 45.0;

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let reading = self.cycle.push(bar[0]);
        self.seen += 1;
        let reading = reading?;
        let degrees = self.phase.push(&self.cycle, reading.smooth_period);
        (self.seen > LOOKBACK)
            .then_some([(degrees * DEGREE).sin(), ((degrees + LEAD) * DEGREE).sin()])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HtSine;

pub type HtSineStream = BarStream<HtSine, 1, 2>;

impl Kernel<1, 2> for HtSine {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["ht_sine_sine", "ht_sine_lead_sine"];

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
