use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "stddev";

pub const PERIOD_DEFAULT: usize = 5;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const NBDEV_DEFAULT: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub nbdev: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            nbdev: NBDEV_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    window: RollingWindow,
    nbdev: f64,
}

/// Population variance, as the sum of squared distances from the window's own
/// mean. Taking the mean of the squares and subtracting the square of the mean
/// is one pass cheaper and drifts ten times further from the oracle, which is
/// outside the tolerance this project allows.
fn variance(mean: f64, values: impl Iterator<Item = f64>, period: f64) -> f64 {
    values
        .map(|value| {
            let distance = value - mean;
            distance * distance
        })
        .sum::<f64>()
        / period
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.window.push(bar[0]) {
            return None;
        }
        let mean = self.window.exact_mean();
        let period = self.window.period() as f64;
        let variance = variance(mean, self.window.values(), period);
        Some([variance.sqrt() * self.nbdev])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.window.preview_is_full() {
            return None;
        }
        let mean = self.window.preview_mean(bar[0]);
        let period = self.window.period() as f64;
        let variance = variance(mean, self.window.preview_values(bar[0]), period);
        Some([variance.sqrt() * self.nbdev])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stddev;

pub type StddevStream = BarStream<Stddev, 1, 1>;

impl Kernel<1, 1> for Stddev {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["stddev"];

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
        if !params.nbdev.is_finite() {
            return Err(TlError::param_out_of_range(
                NAME,
                "nbdev",
                params.nbdev,
                f64::NEG_INFINITY,
                f64::INFINITY,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            window: RollingWindow::new(params.period),
            nbdev: params.nbdev,
        }
    }
}
