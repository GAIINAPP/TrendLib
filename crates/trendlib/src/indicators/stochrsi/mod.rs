use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage, Stochastic};
use crate::indicators::rsi;

pub const NAME: &str = "stochrsi";

pub const PERIOD_DEFAULT: usize = 14;
pub const FASTK_PERIOD_DEFAULT: usize = 5;
pub const FASTD_PERIOD_DEFAULT: usize = 3;
pub const PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
    pub fastk_period: usize,
    pub fastd_period: usize,
    pub fastd_ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            fastk_period: FASTK_PERIOD_DEFAULT,
            fastd_period: FASTD_PERIOD_DEFAULT,
            fastd_ma_type: MaType::Sma,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    rsi: rsi::State,
    raw: Stochastic,
    smoothed: MovingAverage,
}

impl Step<1, 2> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let [rsi] = self.rsi.push(bar)?;
        // The range is taken over the RSI itself, so it is its own high, low
        // and close.
        let k = self.raw.push(rsi, rsi, rsi)?;
        self.smoothed.push(k).map(|d| [k, d])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 2]> {
        let [rsi] = self.rsi.preview(bar)?;
        let k = self.raw.preview(rsi, rsi, rsi)?;
        self.smoothed.preview(k).map(|d| [k, d])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stochrsi;

pub type StochrsiStream = BarStream<Stochrsi, 1, 2>;

impl Kernel<1, 2> for Stochrsi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 2] = ["stochrsi_k", "stochrsi_d"];

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
        for (name, value) in [
            ("fastk_period", params.fastk_period),
            ("fastd_period", params.fastd_period),
        ] {
            if !(FAST_PERIOD_MIN..=PERIOD_MAX).contains(&value) {
                return Err(TlError::param_out_of_range(
                    NAME,
                    name,
                    value,
                    FAST_PERIOD_MIN,
                    PERIOD_MAX,
                ));
            }
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        <rsi::Rsi as Kernel<1, 1>>::lookback(&rsi::Params {
            period: params.period,
        }) + params.fastk_period.saturating_sub(1)
            + params.fastd_ma_type.lookback(params.fastd_period)
    }

    fn state(params: &Params) -> State {
        State {
            rsi: <rsi::Rsi as Kernel<1, 1>>::state(&rsi::Params {
                period: params.period,
            }),
            raw: Stochastic::new(params.fastk_period),
            smoothed: params.fastd_ma_type.state(params.fastd_period),
        }
    }
}
