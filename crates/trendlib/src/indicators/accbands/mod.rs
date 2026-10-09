use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingMean;

pub const NAME: &str = "accbands";

pub const PERIOD_DEFAULT: usize = 20;
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
    upper: RollingMean,
    middle: RollingMean,
    lower: RollingMean,
}

/// How far the bar's own range widens the band, as a fraction of its midpoint.
/// A bar whose high and low sum to zero has no midpoint to measure against and
/// the row is left non-finite rather than guarded.
fn widths(high: f64, low: f64) -> (f64, f64) {
    let ratio = 4.0 * (high - low) / (high + low);
    (high * (1.0 + ratio), low * (1.0 - ratio))
}

impl Step<3, 3> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 3]> {
        let (wide, narrow) = widths(bar[0], bar[1]);
        let upper = self.upper.push(wide);
        let middle = self.middle.push(bar[2]);
        let lower = self.lower.push(narrow)?;
        Some([upper?, middle?, lower])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 3]> {
        let (wide, narrow) = widths(bar[0], bar[1]);
        Some([
            self.upper.preview(wide)?,
            self.middle.preview(bar[2])?,
            self.lower.preview(narrow)?,
        ])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Accbands;

pub type AccbandsStream = BarStream<Accbands, 3, 3>;

impl Kernel<3, 3> for Accbands {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 3] = ["accbands_upper", "accbands_middle", "accbands_lower"];

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
            upper: RollingMean::new(params.period),
            middle: RollingMean::new(params.period),
            lower: RollingMean::new(params.period),
        }
    }
}
