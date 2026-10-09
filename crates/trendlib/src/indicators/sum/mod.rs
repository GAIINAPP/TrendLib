use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "sum";

pub const PERIOD_DEFAULT: usize = 30;
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
pub struct State(RollingWindow);

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.0.push(bar[0]) {
            return None;
        }
        Some([self.0.sum()])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.0.preview_is_full() {
            return None;
        }
        Some([self.0.preview_sum(bar[0])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sum;

pub type SumStream = BarStream<Sum, 1, 1>;

impl Kernel<1, 1> for Sum {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["sum"];

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
        State(RollingWindow::new(params.period))
    }
}
