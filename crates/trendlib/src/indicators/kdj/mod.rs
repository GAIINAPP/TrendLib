use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage, Stochastic};

pub const NAME: &str = "kdj";

pub const FASTK_PERIOD_DEFAULT: usize = 9;
pub const FASTK_PERIOD_MIN: usize = 1;
pub const FASTK_PERIOD_MAX: usize = 100000;
pub const SLOWK_PERIOD_DEFAULT: usize = 3;
pub const SLOWK_PERIOD_MIN: usize = 1;
pub const SLOWK_PERIOD_MAX: usize = 100000;
pub const SLOWK_MA_TYPE_DEFAULT: MaType = MaType::Rma;
pub const SLOWD_PERIOD_DEFAULT: usize = 3;
pub const SLOWD_PERIOD_MIN: usize = 1;
pub const SLOWD_PERIOD_MAX: usize = 100000;
pub const SLOWD_MA_TYPE_DEFAULT: MaType = MaType::Rma;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fastk_period: usize,
    pub slowk_period: usize,
    pub slowk_ma_type: MaType,
    pub slowd_period: usize,
    pub slowd_ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fastk_period: FASTK_PERIOD_DEFAULT,
            slowk_period: SLOWK_PERIOD_DEFAULT,
            slowk_ma_type: SLOWK_MA_TYPE_DEFAULT,
            slowd_period: SLOWD_PERIOD_DEFAULT,
            slowd_ma_type: SLOWD_MA_TYPE_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    raw: Stochastic,
    slowk: MovingAverage,
    slowd: MovingAverage,
}

/// The third line is a stretched version of the first two, so it leaves the
/// 0 to 100 range the others stay inside.
fn lines(k: f64, d: f64) -> [f64; 3] {
    [k, d, 3.0 * k - 2.0 * d]
}

impl Step<3, 3> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 3]> {
        let fast = self.raw.push(bar[0], bar[1], bar[2])?;
        let k = self.slowk.push(fast)?;
        self.slowd.push(k).map(|d| lines(k, d))
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 3]> {
        let fast = self.raw.preview(bar[0], bar[1], bar[2])?;
        let k = self.slowk.preview(fast)?;
        self.slowd.preview(k).map(|d| lines(k, d))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Kdj;

pub type KdjStream = BarStream<Kdj, 3, 3>;

impl Kernel<3, 3> for Kdj {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 3] = ["kdj_k", "kdj_d", "kdj_j"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FASTK_PERIOD_MIN..=FASTK_PERIOD_MAX).contains(&params.fastk_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "fastk_period",
                params.fastk_period,
                FASTK_PERIOD_MIN,
                FASTK_PERIOD_MAX,
            ));
        }
        if !(SLOWK_PERIOD_MIN..=SLOWK_PERIOD_MAX).contains(&params.slowk_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slowk_period",
                params.slowk_period,
                SLOWK_PERIOD_MIN,
                SLOWK_PERIOD_MAX,
            ));
        }
        if !(SLOWD_PERIOD_MIN..=SLOWD_PERIOD_MAX).contains(&params.slowd_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slowd_period",
                params.slowd_period,
                SLOWD_PERIOD_MIN,
                SLOWD_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.fastk_period.saturating_sub(1)
            + params.slowk_ma_type.lookback(params.slowk_period)
            + params.slowd_ma_type.lookback(params.slowd_period)
    }

    fn state(params: &Params) -> State {
        State {
            raw: Stochastic::new(params.fastk_period),
            slowk: params.slowk_ma_type.state(params.slowk_period),
            slowd: params.slowd_ma_type.state(params.slowd_period),
        }
    }
}
