use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "efi";

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
pub struct State {
    previous: Option<f64>,
    smoothed: Ema,
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let previous = self.previous.replace(bar[0])?;
        self.smoothed
            .push((bar[0] - previous) * bar[1])
            .map(|value| [value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let previous = self.previous?;
        self.smoothed
            .preview((bar[0] - previous) * bar[1])
            .map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Efi;

pub type EfiStream = BarStream<Efi, 2, 1>;

impl Kernel<2, 1> for Efi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["efi"];

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
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            smoothed: Ema::new(params.period),
        }
    }
}
