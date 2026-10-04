use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaPair, MaType};

pub const NAME: &str = "ppo";

pub const FAST_PERIOD_DEFAULT: usize = 12;
pub const SLOW_PERIOD_DEFAULT: usize = 26;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
    pub ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
            ma_type: MaType::Ema,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    averages: MaPair,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.averages.push(bar[0]).map(combine)
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.averages.preview(bar[0]).map(combine)
    }
}

/// A slow average of exactly zero has no percentage to report, and TA-Lib
/// answers `0.0` rather than an infinity. The test is exact: a tiny non-zero
/// average is a real, very large percentage, not a division to be guarded.
fn combine((fast, slow): (f64, f64)) -> [f64; 1] {
    if slow == 0.0 {
        [0.0]
    } else {
        [(fast - slow) / slow * 100.0]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ppo;

pub type PpoStream = BarStream<Ppo, 1, 1>;

impl Kernel<1, 1> for Ppo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ppo"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value) in [
            ("fast_period", params.fast_period),
            ("slow_period", params.slow_period),
        ] {
            if !(PERIOD_MIN..=PERIOD_MAX).contains(&value) {
                return Err(TlError::param_out_of_range(
                    NAME, name, value, PERIOD_MIN, PERIOD_MAX,
                ));
            }
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        MaPair::lookback(params.ma_type, params.fast_period, params.slow_period)
    }

    fn state(params: &Params) -> State {
        State {
            averages: MaPair::new(params.ma_type, params.fast_period, params.slow_period),
        }
    }
}
