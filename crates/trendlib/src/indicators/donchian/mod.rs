use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "donchian";

pub const PERIOD_DEFAULT: usize = 20;
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
}

fn channel(top: f64, bottom: f64) -> [f64; 3] {
    [top, (top + bottom) / 2.0, bottom]
}

impl Step<2, 3> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 3]> {
        let top = self.highest.push(bar[0]);
        let bottom = self.lowest.push(bar[1])?;
        Some(channel(top?, bottom))
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 3]> {
        let top = self.highest.preview(bar[0])?;
        let bottom = self.lowest.preview(bar[1])?;
        Some(channel(top, bottom))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Donchian;

pub type DonchianStream = BarStream<Donchian, 2, 3>;

impl Kernel<2, 3> for Donchian {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 3] = ["donchian_upper", "donchian_middle", "donchian_lower"];

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
            highest: RollingExtreme::highest(params.period),
            lowest: RollingExtreme::lowest(params.period),
        }
    }
}
