use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "cmou";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 2;
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
    previous: Option<f64>,
    gains: RollingWindow,
    losses: RollingWindow,
}

fn split(value: f64, previous: f64) -> (f64, f64) {
    let change = value - previous;
    if change > 0.0 {
        (change, 0.0)
    } else {
        (0.0, -change)
    }
}

/// A window that did not move has no net share to report, and TA-Lib answers
/// zero rather than dividing. The test is exact.
fn share(up: f64, down: f64) -> f64 {
    let total = up + down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (up - down) / total
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous.replace(bar[0])?;
        let (gain, loss) = split(bar[0], previous);
        let full = self.gains.push(gain);
        self.losses.push(loss);
        full.then(|| [share(self.gains.sum(), self.losses.sum())])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous?;
        let (gain, loss) = split(bar[0], previous);
        self.gains.preview_is_full().then(|| {
            [share(
                self.gains.preview_sum(gain),
                self.losses.preview_sum(loss),
            )]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cmou;

pub type CmouStream = BarStream<Cmou, 1, 1>;

impl Kernel<1, 1> for Cmou {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["cmou"];

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
            gains: RollingWindow::new(params.period),
            losses: RollingWindow::new(params.period),
        }
    }
}
