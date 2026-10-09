use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "avgdev";

pub const PERIOD_DEFAULT: usize = 14;
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
        let mean = self.0.exact_mean();
        let period = self.0.period() as f64;
        Some([self.0.values().map(|v| (v - mean).abs()).sum::<f64>() / period])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.0.preview_is_full() {
            return None;
        }
        let mean = self.0.preview_mean(bar[0]);
        let period = self.0.period() as f64;
        Some([self
            .0
            .preview_values(bar[0])
            .map(|v| (v - mean).abs())
            .sum::<f64>()
            / period])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Avgdev;

pub type AvgdevStream = BarStream<Avgdev, 1, 1>;

impl Kernel<1, 1> for Avgdev {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["avgdev"];

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
