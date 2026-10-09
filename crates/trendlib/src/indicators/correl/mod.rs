use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Paired, Sums};

pub const NAME: &str = "correl";

pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100000;

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
pub struct State(Paired);

/// A window in which either series never moves has no correlation to report,
/// and TA-Lib answers zero rather than dividing. The product of the two
/// variances is tested against zero exactly, as TA-Lib tests it.
fn correlation(sums: Sums) -> f64 {
    let spread = sums.first_spread * sums.second_spread;
    if spread <= 0.0 {
        0.0
    } else {
        sums.shared / spread.sqrt()
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.0.push(bar[0], bar[1]).map(|sums| [correlation(sums)])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.0
            .preview(bar[0], bar[1])
            .map(|sums| [correlation(sums)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Correl;

pub type CorrelStream = BarStream<Correl, 2, 1>;

impl Kernel<2, 1> for Correl {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["source0", "source1"];
    const OUTPUTS: [&'static str; 1] = ["correl"];

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
        State(Paired::new(params.period))
    }
}
