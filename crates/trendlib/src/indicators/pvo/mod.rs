use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaPair, MaType};

pub const NAME: &str = "pvo";

pub const FAST_PERIOD_DEFAULT: usize = 12;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 26;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;
pub const MA_TYPE_DEFAULT: MaType = MaType::Ema;

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
            ma_type: MA_TYPE_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State(MaPair);

/// A slow average of exactly zero has no percentage to report, and TA-Lib
/// answers `0.0` rather than dividing. The test is exact, as in `ppo`.
fn combine((fast, slow): (f64, f64)) -> [f64; 1] {
    if slow == 0.0 {
        [0.0]
    } else {
        [(fast - slow) / slow * 100.0]
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.0.push(bar[0]).map(combine)
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.0.preview(bar[0]).map(combine)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pvo;

pub type PvoStream = BarStream<Pvo, 1, 1>;

impl Kernel<1, 1> for Pvo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["volume"];
    const OUTPUTS: [&'static str; 1] = ["pvo"];

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
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        MaPair::lookback(params.ma_type, params.fast_period, params.slow_period)
    }

    fn state(params: &Params) -> State {
        State(MaPair::new(
            params.ma_type,
            params.fast_period,
            params.slow_period,
        ))
    }
}
