use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "imi";

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
    gains: RollingWindow,
    losses: RollingWindow,
}

fn split(bar: [f64; 2]) -> (f64, f64) {
    let (open, close) = (bar[0], bar[1]);
    if close > open {
        (close - open, 0.0)
    } else if close < open {
        (0.0, open - close)
    } else {
        (0.0, 0.0)
    }
}

/// A window of bars that all closed where they opened splits evenly: TA-Lib
/// answers 50 rather than dividing. The test is exact.
fn share(up: f64, down: f64) -> f64 {
    let total = up + down;
    if total == 0.0 {
        50.0
    } else {
        100.0 * up / total
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let (gain, loss) = split(bar);
        let full = self.gains.push(gain);
        self.losses.push(loss);
        full.then(|| [share(self.gains.values().sum(), self.losses.values().sum())])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let (gain, loss) = split(bar);
        self.gains.preview_is_full().then(|| {
            [share(
                self.gains.preview_values(gain).sum(),
                self.losses.preview_values(loss).sum(),
            )]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Imi;

pub type ImiStream = BarStream<Imi, 2, 1>;

impl Kernel<2, 1> for Imi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["open", "close"];
    const OUTPUTS: [&'static str; 1] = ["imi"];

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
            gains: RollingWindow::new(params.period),
            losses: RollingWindow::new(params.period),
        }
    }
}
