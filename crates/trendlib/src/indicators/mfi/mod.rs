use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "mfi";

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
pub struct State {
    previous: Option<f64>,
    up: RollingWindow,
    down: RollingWindow,
}

/// A bar that closed where the last one did moves no money either way, so it
/// adds nothing to both sides rather than being split between them.
fn split(typical: f64, previous: f64, flow: f64) -> (f64, f64) {
    if typical > previous {
        (flow, 0.0)
    } else if typical < previous {
        (0.0, flow)
    } else {
        (0.0, 0.0)
    }
}

/// With no flow at all there is no share to report, and TA-Lib answers zero
/// rather than dividing. The test is exact.
fn share(up: f64, down: f64) -> f64 {
    let total = up + down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (up / total)
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let typical = (bar[0] + bar[1] + bar[2]) / 3.0;
        let previous = self.previous.replace(typical)?;
        let (up, down) = split(typical, previous, typical * bar[3]);
        let full = self.up.push(up);
        self.down.push(down);
        full.then(|| {
            [share(
                self.up.values().sum::<f64>(),
                self.down.values().sum::<f64>(),
            )]
        })
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let typical = (bar[0] + bar[1] + bar[2]) / 3.0;
        let previous = self.previous?;
        let (up, down) = split(typical, previous, typical * bar[3]);
        self.up.preview_is_full().then(|| {
            [share(
                self.up.preview_values(up).sum::<f64>(),
                self.down.preview_values(down).sum::<f64>(),
            )]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mfi;

pub type MfiStream = BarStream<Mfi, 4, 1>;

impl Kernel<4, 1> for Mfi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["high", "low", "close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["mfi"];

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

    // The flows are differences, so one more bar than there are changes.
    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            up: RollingWindow::new(params.period),
            down: RollingWindow::new(params.period),
        }
    }
}
