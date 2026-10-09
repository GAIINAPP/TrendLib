use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "percentile";

pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PERCENTILE_DEFAULT: f64 = 50.0;
pub const PERCENTILE_MIN: f64 = 0.0;
pub const PERCENTILE_MAX: f64 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub percentile: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            percentile: PERCENTILE_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    window: RollingWindow,
    percentile: f64,
    /// Scratch space for the sorted window, kept so a batch run does not
    /// allocate once per bar.
    sorted: Vec<f64>,
}

impl State {
    /// Nearest rank: the smallest value at or above the requested share of the
    /// window. The rank is rounded up, so any percentile above zero names at
    /// least the first value and 100 names the last.
    fn at(&mut self, values: impl Iterator<Item = f64>) -> f64 {
        self.sorted.clear();
        self.sorted.extend(values);
        self.sorted.sort_by(f64::total_cmp);
        let count = self.sorted.len();
        let rank = (count as f64 * self.percentile / 100.0).ceil() as usize;
        self.sorted[rank.max(1) - 1]
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.window.push(bar[0]) {
            return None;
        }
        let window: Vec<f64> = self.window.values().collect();
        Some([self.at(window.into_iter())])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.window.preview_is_full() {
            return None;
        }
        let mut scratch = self.clone();
        let window: Vec<f64> = self.window.preview_values(bar[0]).collect();
        Some([scratch.at(window.into_iter())])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Percentile;

pub type PercentileStream = BarStream<Percentile, 1, 1>;

impl Kernel<1, 1> for Percentile {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["percentile"];

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
        if !(PERCENTILE_MIN..=PERCENTILE_MAX).contains(&params.percentile) {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "percentile",
                params.percentile,
                PERCENTILE_MIN,
                PERCENTILE_MAX,
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
            percentile: params.percentile,
            sorted: Vec::with_capacity(params.period),
        }
    }
}
