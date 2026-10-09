use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "eri";

pub const PERIOD_DEFAULT: usize = 13;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100000;

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
pub struct State(Ema);

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let average = self.0.push(bar[2])?;
        Some([bar[0] - average, bar[1] - average])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let average = self.0.preview(bar[2])?;
        Some([bar[0] - average, bar[1] - average])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Eri;

pub type EriStream = BarStream<Eri, 3, 2>;

impl Kernel<3, 2> for Eri {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["eri_bull_power", "eri_bear_power"];

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
        State(Ema::new(params.period))
    }
}
