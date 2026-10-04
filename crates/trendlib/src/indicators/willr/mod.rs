use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "willr";

pub const PERIOD_DEFAULT: usize = 14;
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
    highest: RollingExtreme,
    lowest: RollingExtreme,
}

// A range of zero leaves the close nowhere in particular, and the position is
// reported as 0 rather than as a division by zero. TA-Lib answers the same.
fn position(highest: f64, lowest: f64, close: f64) -> f64 {
    let span = highest - lowest;
    if span == 0.0 {
        0.0
    } else {
        -100.0 * (highest - close) / span
    }
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let highest = self.highest.push(bar[0]);
        let lowest = self.lowest.push(bar[1]);
        Some([position(highest?, lowest?, bar[2])])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let highest = self.highest.preview(bar[0])?;
        let lowest = self.lowest.preview(bar[1])?;
        Some([position(highest, lowest, bar[2])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Willr;

pub type WillrStream = BarStream<Willr, 3, 1>;

impl Kernel<3, 1> for Willr {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["willr"];

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
            highest: RollingExtreme::highest(params.period),
            lowest: RollingExtreme::lowest(params.period),
        }
    }
}
