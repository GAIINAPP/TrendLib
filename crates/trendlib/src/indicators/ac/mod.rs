use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingMean;

pub const NAME: &str = "ac";

pub const FAST_PERIOD_DEFAULT: usize = 5;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 34;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;
pub const SIGNAL_PERIOD_DEFAULT: usize = 5;
pub const SIGNAL_PERIOD_MIN: usize = 2;
pub const SIGNAL_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    fast: RollingMean,
    slow: RollingMean,
    signal: RollingMean,
}

fn midpoint(bar: [f64; 2]) -> f64 {
    (bar[0] + bar[1]) / 2.0
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let value = midpoint(bar);
        let fast = self.fast.push(value);
        let slow = self.slow.push(value)?;
        let oscillator = fast? - slow;
        self.signal.push(oscillator).map(|mean| [oscillator - mean])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let value = midpoint(bar);
        let oscillator = self.fast.preview(value)? - self.slow.preview(value)?;
        self.signal
            .preview(oscillator)
            .map(|mean| [oscillator - mean])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ac;

pub type AcStream = BarStream<Ac, 2, 1>;

impl Kernel<2, 1> for Ac {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["ac"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
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
        params.fast_period.max(params.slow_period).saturating_sub(1)
            + params.signal_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            fast: RollingMean::new(params.fast_period),
            slow: RollingMean::new(params.slow_period),
            signal: RollingMean::new(params.signal_period),
        }
    }
}
