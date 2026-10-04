use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "cci";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

/// Lambert's constant, chosen so that most readings fall between -100 and 100.
const SCALE: f64 = 0.015;

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
    typical: RollingWindow,
}

fn typical_price(bar: [f64; 3]) -> f64 {
    (bar[0] + bar[1] + bar[2]) / 3.0
}

// A window that never moved has no mean deviation to measure against, so the
// index is reported as 0 rather than as a division by zero, as TA-Lib does.
fn index(now: f64, mean: f64, deviation: f64) -> f64 {
    if deviation == 0.0 {
        0.0
    } else {
        (now - mean) / (SCALE * deviation)
    }
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let now = typical_price(bar);
        if !self.typical.push(now) {
            return None;
        }
        let mean = self.typical.exact_mean();
        let period = self.typical.period() as f64;
        let deviation = self.typical.values().map(|v| (v - mean).abs()).sum::<f64>() / period;
        Some([index(now, mean, deviation)])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let now = typical_price(bar);
        if !self.typical.preview_is_full() {
            return None;
        }
        let mean = self.typical.preview_mean(now);
        let period = self.typical.period() as f64;
        let deviation = self
            .typical
            .preview_values(now)
            .map(|v| (v - mean).abs())
            .sum::<f64>()
            / period;
        Some([index(now, mean, deviation)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cci;

pub type CciStream = BarStream<Cci, 3, 1>;

impl Kernel<3, 1> for Cci {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cci"];

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
            typical: RollingWindow::new(params.period),
        }
    }
}
