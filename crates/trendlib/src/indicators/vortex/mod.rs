use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{RollingWindow, TrueRange};

pub const NAME: &str = "vortex";

pub const PERIOD_DEFAULT: usize = 14;
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
    previous: Option<(f64, f64)>,
    range: TrueRange,
    up: RollingWindow,
    down: RollingWindow,
    ranges: RollingWindow,
}

/// A window with no true range at all has no share to report, and TA-Lib
/// answers zero rather than dividing. The test is exact.
fn share(moved: f64, total: f64) -> f64 {
    if total == 0.0 { 0.0 } else { moved / total }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let range = self.range.push(bar[0], bar[1], bar[2]);
        let previous = self.previous.replace((bar[0], bar[1]));
        let (Some(range), Some((previous_high, previous_low))) = (range, previous) else {
            return None;
        };
        let full = self.up.push((bar[0] - previous_low).abs());
        self.down.push((bar[1] - previous_high).abs());
        self.ranges.push(range);
        full.then(|| {
            let total = self.ranges.sum();
            [share(self.up.sum(), total), share(self.down.sum(), total)]
        })
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let range = self.range.preview(bar[0], bar[1])?;
        let (previous_high, previous_low) = self.previous?;
        self.up.preview_is_full().then(|| {
            let total = self.ranges.preview_sum(range);
            [
                self.up.preview_sum((bar[0] - previous_low).abs()) / total,
                self.down.preview_sum((bar[1] - previous_high).abs()) / total,
            ]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vortex;

pub type VortexStream = BarStream<Vortex, 3, 2>;

impl Kernel<3, 2> for Vortex {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["vortex_plusvi", "vortex_minusvi"];

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
            range: TrueRange::new(),
            up: RollingWindow::new(params.period),
            down: RollingWindow::new(params.period),
            ranges: RollingWindow::new(params.period),
        }
    }
}
