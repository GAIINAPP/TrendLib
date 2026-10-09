use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{RollingWindow, Wilder};

pub const NAME: &str = "rvi";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100000;
pub const STDDEV_PERIOD_DEFAULT: usize = 10;
pub const STDDEV_PERIOD_MIN: usize = 2;
pub const STDDEV_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
    pub stddev_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            stddev_period: STDDEV_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    window: RollingWindow,
    previous: Option<f64>,
    up: Wilder,
    down: Wilder,
}

/// Population standard deviation about the window's own mean, as `stddev`
/// takes it.
fn deviation(window: &RollingWindow, values: impl Iterator<Item = f64>, mean: f64) -> f64 {
    let period = window.period() as f64;
    let variance = values
        .map(|value| {
            let distance = value - mean;
            distance * distance
        })
        .sum::<f64>()
        / period;
    variance.sqrt()
}

/// A stretch with no volatility at all splits evenly: TA-Lib answers 50 rather
/// than dividing, as `imi` does. The test is exact.
fn share(up: f64, down: f64) -> f64 {
    let total = up + down;
    if total == 0.0 {
        50.0
    } else {
        100.0 * up / total
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let full = self.window.push(bar[0]);
        let previous = self.previous.replace(bar[0]);
        if !full {
            return None;
        }
        let mean = self.window.exact_mean();
        let spread = deviation(&self.window, self.window.values(), mean);
        let rose = previous.is_some_and(|previous| bar[0] > previous);
        let fell = previous.is_some_and(|previous| bar[0] < previous);
        // Both averages must be offered the bar before either `?` can bail.
        let up = self.up.push(if rose { spread } else { 0.0 });
        let down = self.down.push(if fell { spread } else { 0.0 });
        Some([share(up?, down?)])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        if !self.window.preview_is_full() {
            return None;
        }
        let mean = self.window.preview_mean(bar[0]);
        let spread = deviation(&self.window, self.window.preview_values(bar[0]), mean);
        let previous = self.previous?;
        let up = self
            .up
            .preview(if bar[0] > previous { spread } else { 0.0 })?;
        let down = self
            .down
            .preview(if bar[0] < previous { spread } else { 0.0 })?;
        Some([share(up, down)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rvi;

pub type RviStream = BarStream<Rvi, 1, 1>;

impl Kernel<1, 1> for Rvi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["rvi"];

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
        if !(STDDEV_PERIOD_MIN..=STDDEV_PERIOD_MAX).contains(&params.stddev_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "stddev_period",
                params.stddev_period,
                STDDEV_PERIOD_MIN,
                STDDEV_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.stddev_period.saturating_sub(1) + params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            window: RollingWindow::new(params.stddev_period),
            previous: None,
            up: Wilder::new(params.period),
            down: Wilder::new(params.period),
        }
    }
}
