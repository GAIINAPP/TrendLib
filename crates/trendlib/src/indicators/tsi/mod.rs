use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "tsi";

pub const FIRST_PERIOD_DEFAULT: usize = 25;
pub const FIRST_PERIOD_MIN: usize = 2;
pub const FIRST_PERIOD_MAX: usize = 100000;
pub const SECOND_PERIOD_DEFAULT: usize = 13;
pub const SECOND_PERIOD_MIN: usize = 2;
pub const SECOND_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub first_period: usize,
    pub second_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            first_period: FIRST_PERIOD_DEFAULT,
            second_period: SECOND_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<f64>,
    moves: [Ema; 2],
    sizes: [Ema; 2],
}

/// A series that never moves has no movement to take a share of, and TA-Lib
/// answers zero rather than dividing. The test is exact.
fn share(moved: f64, size: f64) -> f64 {
    if size == 0.0 {
        0.0
    } else {
        100.0 * moved / size
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let change = bar[0] - self.previous.replace(bar[0])?;
        // Both chains must be offered the bar before either `?` can bail, or
        // the one that is still warming up would miss what the other consumed.
        let moved = self.moves[0]
            .push(change)
            .and_then(|once| self.moves[1].push(once));
        let size = self.sizes[0]
            .push(change.abs())
            .and_then(|once| self.sizes[1].push(once));
        Some([share(moved?, size?)])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let change = bar[0] - self.previous?;
        let moved = self.moves[1].preview(self.moves[0].preview(change)?)?;
        let size = self.sizes[1].preview(self.sizes[0].preview(change.abs())?)?;
        Some([share(moved, size)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tsi;

pub type TsiStream = BarStream<Tsi, 1, 1>;

impl Kernel<1, 1> for Tsi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["tsi"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FIRST_PERIOD_MIN..=FIRST_PERIOD_MAX).contains(&params.first_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "first_period",
                params.first_period,
                FIRST_PERIOD_MIN,
                FIRST_PERIOD_MAX,
            ));
        }
        if !(SECOND_PERIOD_MIN..=SECOND_PERIOD_MAX).contains(&params.second_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "second_period",
                params.second_period,
                SECOND_PERIOD_MIN,
                SECOND_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.first_period + params.second_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            moves: [
                Ema::new(params.first_period),
                Ema::new(params.second_period),
            ],
            sizes: [
                Ema::new(params.first_period),
                Ema::new(params.second_period),
            ],
        }
    }
}
