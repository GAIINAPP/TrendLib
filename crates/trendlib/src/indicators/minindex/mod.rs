use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "minindex";

pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 2;
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
    extreme: RollingExtreme,
    offset: f64,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.extreme.push(bar[0])?;
        self.extreme
            .best_row()
            .map(|row| [row as f64 + self.offset])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let since = self.extreme.preview_bars_since_best(bar[0])?;
        Some([self.extreme.seen() as f64 - since as f64 + self.offset])
    }

    fn start_at(&mut self, row: usize) {
        self.offset = row as f64;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Minindex;

pub type MinindexStream = BarStream<Minindex, 1, 1>;

impl Kernel<1, 1> for Minindex {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["minindex"];

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
            extreme: RollingExtreme::lowest(params.period),
            offset: 0.0,
        }
    }
}
