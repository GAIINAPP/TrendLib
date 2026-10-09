use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "minmaxindex";

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
    lowest: RollingExtreme,
    highest: RollingExtreme,
    offset: f64,
}

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        self.lowest.push(bar[0]);
        self.highest.push(bar[0]);
        let low = self.lowest.best_row()?;
        let high = self.highest.best_row()?;
        Some([low as f64 + self.offset, high as f64 + self.offset])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let seen = self.lowest.seen() as f64;
        let low = self.lowest.preview_bars_since_best(bar[0])?;
        let high = self.highest.preview_bars_since_best(bar[0])?;
        Some([
            seen - low as f64 + self.offset,
            seen - high as f64 + self.offset,
        ])
    }

    fn start_at(&mut self, row: usize) {
        self.offset = row as f64;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Minmaxindex;

pub type MinmaxindexStream = BarStream<Minmaxindex, 1, 2>;

impl Kernel<1, 2> for Minmaxindex {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["minmaxindex_min_index", "minmaxindex_max_index"];

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
            lowest: RollingExtreme::lowest(params.period),
            highest: RollingExtreme::highest(params.period),
            offset: 0.0,
        }
    }
}
