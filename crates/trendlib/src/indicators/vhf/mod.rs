use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "vhf";

pub const PERIOD_DEFAULT: usize = 28;
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
    window: RollingWindow,
    moves: RollingWindow,
    previous: Option<f64>,
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let moved = self.previous.replace(bar[0]).map(|p| (bar[0] - p).abs());
        self.window.push(bar[0]);
        // The first bar has no change to record, so the window of changes is
        // one bar behind the window of values and is what decides when the
        // first row appears.
        let full = moved.is_some_and(|moved| self.moves.push(moved));
        full.then(|| [ratio(span(&self.window), self.moves.values().sum())])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let moved = (bar[0] - self.previous?).abs();
        self.moves.preview_is_full().then(|| {
            let held: Vec<f64> = self.window.preview_values(bar[0]).collect();
            let top = held.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let bottom = held.iter().copied().fold(f64::INFINITY, f64::min);
            [ratio(top - bottom, self.moves.preview_values(moved).sum())]
        })
    }
}

/// A window that did not move has no distance to measure its range against,
/// and TA-Lib answers zero rather than dividing. The test is exact.
fn ratio(span: f64, travelled: f64) -> f64 {
    if travelled == 0.0 {
        0.0
    } else {
        span / travelled
    }
}

fn span(window: &RollingWindow) -> f64 {
    let top = window.values().fold(f64::NEG_INFINITY, f64::max);
    let bottom = window.values().fold(f64::INFINITY, f64::min);
    top - bottom
}

#[derive(Clone, Copy, Debug)]
pub struct Vhf;

pub type VhfStream = BarStream<Vhf, 1, 1>;

impl Kernel<1, 1> for Vhf {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["vhf"];

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
            window: RollingWindow::new(params.period),
            moves: RollingWindow::new(params.period),
            previous: None,
        }
    }
}
