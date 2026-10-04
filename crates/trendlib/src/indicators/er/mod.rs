use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, RollingWindow};

pub const NAME: &str = "er";

pub const PERIOD_DEFAULT: usize = 10;
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
    anchor: Lagged,
    moves: RollingWindow,
    previous: Option<f64>,
}

/// A window that went nowhere at all has no ratio to take, and TA-Lib reads it
/// as fully efficient rather than dividing. `kama`, which adapts on this same
/// ratio, takes the same reading. The test is exact.
fn ratio(change: f64, travelled: f64) -> f64 {
    if travelled == 0.0 {
        1.0
    } else {
        change / travelled
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let anchor = self.anchor.push(bar[0]);
        let moved = self.previous.replace(bar[0]).map(|p| (bar[0] - p).abs());
        let full = self.moves.push(moved.unwrap_or(0.0));
        let anchor = anchor?;
        full.then(|| [ratio((bar[0] - anchor).abs(), self.moves.values().sum())])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let anchor = self.anchor.earlier()?;
        let moved = (bar[0] - self.previous?).abs();
        self.moves.preview_is_full().then(|| {
            [ratio(
                (bar[0] - anchor).abs(),
                self.moves.preview_values(moved).sum(),
            )]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Er;

pub type ErStream = BarStream<Er, 1, 1>;

impl Kernel<1, 1> for Er {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["er"];

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
            anchor: Lagged::new(params.period),
            moves: RollingWindow::new(params.period),
            previous: None,
        }
    }
}
