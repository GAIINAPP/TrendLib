use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage, Stochastic};

pub const NAME: &str = "stochf";

pub const FASTK_PERIOD_DEFAULT: usize = 5;
pub const FASTD_PERIOD_DEFAULT: usize = 3;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fastk_period: usize,
    pub fastd_period: usize,
    pub fastd_ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fastk_period: FASTK_PERIOD_DEFAULT,
            fastd_period: FASTD_PERIOD_DEFAULT,
            fastd_ma_type: MaType::Sma,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    raw: Stochastic,
    smoothed: MovingAverage,
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let k = self.raw.push(bar[0], bar[1], bar[2])?;
        // %K is held back until %D has a value, so the two columns start on
        // the same row. TA-Lib does the same.
        self.smoothed.push(k).map(|d| [k, d])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let k = self.raw.preview(bar[0], bar[1], bar[2])?;
        self.smoothed.preview(k).map(|d| [k, d])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stochf;

pub type StochfStream = BarStream<Stochf, 3, 2>;

impl Kernel<3, 2> for Stochf {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["stochf_k", "stochf_d"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value) in [
            ("fastk_period", params.fastk_period),
            ("fastd_period", params.fastd_period),
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
        params.fastk_period.saturating_sub(1) + params.fastd_ma_type.lookback(params.fastd_period)
    }

    fn state(params: &Params) -> State {
        State {
            raw: Stochastic::new(params.fastk_period),
            smoothed: params.fastd_ma_type.state(params.fastd_period),
        }
    }
}
