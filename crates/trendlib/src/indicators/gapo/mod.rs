use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "gapo";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

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
    highest: RollingExtreme,
    lowest: RollingExtreme,
    scale: f64,
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let top = self.highest.push(bar[0]);
        let bottom = self.lowest.push(bar[1]);
        Some([(top? - bottom?).ln() / self.scale])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let top = self.highest.preview(bar[0])?;
        let bottom = self.lowest.preview(bar[1])?;
        Some([(top - bottom).ln() / self.scale])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gapo;

pub type GapoStream = BarStream<Gapo, 2, 1>;

impl Kernel<2, 1> for Gapo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["gapo"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period - 1
    }

    fn state(params: &Params) -> State {
        State {
            highest: RollingExtreme::highest(params.period),
            lowest: RollingExtreme::lowest(params.period),
            scale: (params.period as f64).ln(),
        }
    }
}
