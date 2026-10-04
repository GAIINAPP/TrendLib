use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::WilderSum;

pub const NAME: &str = "minus_dm";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<(f64, f64)>,
    total: WilderSum,
}

/// A bar moves in one direction only: whichever of the two edges extended
/// further counts, and only if it extended at all. An inside bar counts for
/// neither.
fn movement(previous: (f64, f64), bar: [f64; 2]) -> f64 {
    let up = bar[0] - previous.0;
    let down = previous.1 - bar[1];
    if down > up && down > 0.0 { down } else { 0.0 }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let previous = self.previous.replace((bar[0], bar[1]))?;
        self.total
            .push(movement(previous, bar))
            .map(|value| [value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let previous = self.previous?;
        self.total
            .preview(movement(previous, bar))
            .map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MinusDm;

pub type MinusDmStream = BarStream<MinusDm, 2, 1>;

impl Kernel<2, 1> for MinusDm {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["minus_dm"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            // Seeded over one movement fewer than it decays by, which is
            // what puts the first value a bar earlier than the
            // accumulation alone would.
            total: WilderSum::with_seed(params.period.saturating_sub(1).max(1), params.period),
        }
    }
}
