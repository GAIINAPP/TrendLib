use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;
use crate::indicators::fib_retracement::levels;

pub const NAME: &str = "fib_retracement_window";

pub const PERIOD_DEFAULT: usize = 20;
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

/// The levels of the leg between a window's two extremes. It ends at the one
/// set more recently, and at the high when one bar set both.
fn leg(top: f64, bottom: f64, since_top: u64, since_bottom: u64) -> [f64; 7] {
    if since_top <= since_bottom {
        levels(bottom, top)
    } else {
        levels(top, bottom)
    }
}

#[derive(Clone, Debug)]
pub struct State {
    highest: RollingExtreme,
    lowest: RollingExtreme,
}

impl Step<2, 7> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 7]> {
        let top = self.highest.push(bar[0]);
        let bottom = self.lowest.push(bar[1]);
        Some(leg(
            top?,
            bottom?,
            self.highest.bars_since_best()?,
            self.lowest.bars_since_best()?,
        ))
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 7]> {
        Some(leg(
            self.highest.preview(bar[0])?,
            self.lowest.preview(bar[1])?,
            self.highest.preview_bars_since_best(bar[0])?,
            self.lowest.preview_bars_since_best(bar[1])?,
        ))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FibRetracementWindow;

pub type FibRetracementWindowStream = BarStream<FibRetracementWindow, 2, 7>;

impl Kernel<2, 7> for FibRetracementWindow {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 7] = [
        "fib_window_0",
        "fib_window_236",
        "fib_window_382",
        "fib_window_500",
        "fib_window_618",
        "fib_window_786",
        "fib_window_1000",
    ];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)
    }

    fn lookback(params: &Params) -> usize {
        params.period - 1
    }

    fn state(params: &Params) -> State {
        // Of several bars at the same extreme the most recent one holds it, so
        // which extreme came later depends on the window alone.
        State {
            highest: RollingExtreme::highest(params.period).preferring_recent(),
            lowest: RollingExtreme::lowest(params.period).preferring_recent(),
        }
    }
}
