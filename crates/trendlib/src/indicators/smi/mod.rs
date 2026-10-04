use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Ema, RollingExtreme};

pub const NAME: &str = "smi";

pub const PERIOD_DEFAULT: usize = 13;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100000;
pub const FAST_PERIOD_DEFAULT: usize = 2;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 25;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;
pub const SIGNAL_PERIOD_DEFAULT: usize = 9;
pub const SIGNAL_PERIOD_MIN: usize = 2;
pub const SIGNAL_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    highest: RollingExtreme,
    lowest: RollingExtreme,
    /// The distance from the middle of the range, smoothed twice.
    offset: [Ema; 2],
    /// The range itself, smoothed the same way so the ratio stays bounded.
    span: [Ema; 2],
    signal: Ema,
}

/// A range of exactly zero leaves the close nowhere to sit, and TA-Lib answers
/// zero rather than dividing. The test is exact.
fn index(offset: f64, span: f64) -> f64 {
    if span == 0.0 {
        0.0
    } else {
        100.0 * offset / (span / 2.0)
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let top = self.highest.push(bar[0]);
        let bottom = self.lowest.push(bar[1])?;
        let top = top?;
        let (offset, span) = (bar[2] - (top + bottom) / 2.0, top - bottom);
        // Both chains have to be offered the bar before either can bail.
        let offset = self.offset[0]
            .push(offset)
            .and_then(|once| self.offset[1].push(once));
        let span = self.span[0]
            .push(span)
            .and_then(|once| self.span[1].push(once));
        let value = index(offset?, span?);
        self.signal.push(value).map(|signal| [value, signal])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let top = self.highest.preview(bar[0])?;
        let bottom = self.lowest.preview(bar[1])?;
        let offset =
            self.offset[1].preview(self.offset[0].preview(bar[2] - (top + bottom) / 2.0)?)?;
        let span = self.span[1].preview(self.span[0].preview(top - bottom)?)?;
        let value = index(offset, span);
        self.signal.preview(value).map(|signal| [value, signal])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Smi;

pub type SmiStream = BarStream<Smi, 3, 2>;

impl Kernel<3, 2> for Smi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["smi", "smi_smisignal"];

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
        if !(FAST_PERIOD_MIN..=FAST_PERIOD_MAX).contains(&params.fast_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "fast_period",
                params.fast_period,
                FAST_PERIOD_MIN,
                FAST_PERIOD_MAX,
            ));
        }
        if !(SLOW_PERIOD_MIN..=SLOW_PERIOD_MAX).contains(&params.slow_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slow_period",
                params.slow_period,
                SLOW_PERIOD_MIN,
                SLOW_PERIOD_MAX,
            ));
        }
        if !(SIGNAL_PERIOD_MIN..=SIGNAL_PERIOD_MAX).contains(&params.signal_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "signal_period",
                params.signal_period,
                SIGNAL_PERIOD_MIN,
                SIGNAL_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
            + params.slow_period.saturating_sub(1)
            + params.fast_period.saturating_sub(1)
            + params.signal_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            highest: RollingExtreme::highest(params.period),
            lowest: RollingExtreme::lowest(params.period),
            offset: [Ema::new(params.slow_period), Ema::new(params.fast_period)],
            span: [Ema::new(params.slow_period), Ema::new(params.fast_period)],
            signal: Ema::new(params.signal_period),
        }
    }
}
