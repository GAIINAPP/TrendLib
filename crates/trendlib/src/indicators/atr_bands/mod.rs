use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::atr;

pub const NAME: &str = "atr_bands";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;
pub const SHIFT_DEFAULT: f64 = 3.0;
pub const SHIFT_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub shift: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            shift: SHIFT_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    range: atr::State,
    shift: f64,
}

impl State {
    fn bands(&self, close: f64, average: f64) -> [f64; 2] {
        let width = average * self.shift;
        [close + width, close - width]
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let [average] = self.range.push(bar)?;
        Some(self.bands(bar[2], average))
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let [average] = self.range.preview(bar)?;
        Some(self.bands(bar[2], average))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AtrBands;

pub type AtrBandsStream = BarStream<AtrBands, 3, 2>;

impl Kernel<3, 2> for AtrBands {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["atr_bands_upper", "atr_bands_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "shift", params.shift, SHIFT_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            range: atr::Atr::state(&atr::Params {
                period: params.period,
            }),
            shift: params.shift,
        }
    }
}
