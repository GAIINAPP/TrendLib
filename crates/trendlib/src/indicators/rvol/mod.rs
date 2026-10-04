use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "rvol";

pub const PERIOD_DEFAULT: usize = 20;
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
    /// The `period` bars **before** the current one, so a bar is never part of
    /// the average it is measured against.
    history: RollingWindow,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let out = self.preview(bar);
        self.history.push(bar[0]);
        out
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.history.is_full() {
            return None;
        }
        let mean = self.history.values().sum::<f64>() / self.history.period() as f64;
        Some([bar[0] / mean])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rvol;

pub type RvolStream = BarStream<Rvol, 1, 1>;

impl Kernel<1, 1> for Rvol {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["volume"];
    const OUTPUTS: [&'static str; 1] = ["rvol"];

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
            history: RollingWindow::new(params.period),
        }
    }
}
