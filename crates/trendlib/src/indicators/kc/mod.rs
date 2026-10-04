use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Ema, TrueRange, Wilder};

pub const NAME: &str = "kc";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const ATR_PERIOD_DEFAULT: usize = 10;
pub const ATR_PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;
pub const NBDEV_DEFAULT: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub atr_period: usize,
    pub nbdev: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            atr_period: ATR_PERIOD_DEFAULT,
            nbdev: NBDEV_DEFAULT,
        }
    }
}

/// The row the first band lands on. The middle band needs `period - 1` bars
/// and the range needs `atr_period` of them, one more than it averages because
/// a true range reaches back to the previous close.
fn lookback(params: &Params) -> usize {
    params.period.saturating_sub(1).max(params.atr_period)
}

#[derive(Clone, Debug)]
pub struct State {
    middle: Ema,
    range: TrueRange,
    average_range: Wilder,
    /// True ranges to discard before the average starts, so that its first
    /// value lands on the same bar as the middle band's rather than earlier.
    skip: usize,
    seen: usize,
    nbdev: f64,
}

fn channel(middle: f64, width: f64) -> [f64; 3] {
    [middle + width, middle, middle - width]
}

fn typical(bar: [f64; 3]) -> f64 {
    (bar[0] + bar[1] + bar[2]) / 3.0
}

impl Step<3, 3> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 3]> {
        self.seen += 1;
        let middle = self.middle.push(typical(bar));
        let range = self.range.push(bar[0], bar[1], bar[2]);
        let width = range
            .filter(|_| self.seen > self.skip)
            .and_then(|range| self.average_range.push(range));
        Some(channel(middle?, width? * self.nbdev))
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 3]> {
        let middle = self.middle.preview(typical(bar))?;
        let range = self.range.preview(bar[0], bar[1])?;
        if self.seen < self.skip {
            return None;
        }
        let width = self.average_range.preview(range)?;
        Some(channel(middle, width * self.nbdev))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Kc;

pub type KcStream = BarStream<Kc, 3, 3>;

impl Kernel<3, 3> for Kc {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 3] = ["kc_upper", "kc_middle", "kc_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value, min) in [
            ("period", params.period, PERIOD_MIN),
            ("atr_period", params.atr_period, ATR_PERIOD_MIN),
        ] {
            if !(min..=PERIOD_MAX).contains(&value) {
                return Err(TlError::param_out_of_range(
                    NAME, name, value, min, PERIOD_MAX,
                ));
            }
        }
        if !params.nbdev.is_finite() {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "nbdev",
                params.nbdev,
                f64::NEG_INFINITY,
                f64::INFINITY,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        lookback(params)
    }

    fn state(params: &Params) -> State {
        State {
            middle: Ema::new(params.period),
            range: TrueRange::new(),
            average_range: Wilder::new(params.atr_period),
            // The average wants the `atr_period` true ranges that end on the
            // first band's row, so the ones before that row are passed over.
            skip: lookback(params) - params.atr_period + 1,
            seen: 0,
            nbdev: params.nbdev,
        }
    }
}
