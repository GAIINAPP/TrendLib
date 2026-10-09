use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "fractal_chaos_bands";

pub const LEFT_BARS_DEFAULT: usize = 2;
pub const LEFT_BARS_MIN: usize = 1;
pub const LEFT_BARS_MAX: usize = 100_000;
pub const RIGHT_BARS_DEFAULT: usize = 2;
pub const RIGHT_BARS_MIN: usize = 1;
pub const RIGHT_BARS_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub left_bars: usize,
    pub right_bars: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            left_bars: LEFT_BARS_DEFAULT,
            right_bars: RIGHT_BARS_DEFAULT,
        }
    }
}

/// The bar `left` places into the window, if it beats every other bar there.
fn fractal(window: &RollingWindow, left: usize, beats: fn(f64, f64) -> bool) -> Option<f64> {
    let middle = window.values().nth(left)?;
    window
        .values()
        .enumerate()
        .all(|(offset, value)| offset == left || beats(middle, value))
        .then_some(middle)
}

#[derive(Clone, Debug)]
pub struct State {
    highs: RollingWindow,
    lows: RollingWindow,
    left: usize,
    upper: f64,
    lower: f64,
}

impl State {
    fn advance(&mut self, [high, low]: [f64; 2]) -> Option<[f64; 2]> {
        self.lows.push(low);
        if !self.highs.push(high) {
            return None;
        }
        if let Some(top) = fractal(&self.highs, self.left, |middle, other| middle > other) {
            self.upper = top;
        }
        if let Some(bottom) = fractal(&self.lows, self.left, |middle, other| middle < other) {
            self.lower = bottom;
        }
        Some([self.upper, self.lower])
    }
}

impl Step<2, 2> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FractalChaosBands;

pub type FractalChaosBandsStream = BarStream<FractalChaosBands, 2, 2>;

impl Kernel<2, 2> for FractalChaosBands {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 2] = ["fractal_chaos_upper", "fractal_chaos_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "left_bars",
            params.left_bars,
            LEFT_BARS_MIN,
            LEFT_BARS_MAX,
        )?;
        whole(
            NAME,
            "right_bars",
            params.right_bars,
            RIGHT_BARS_MIN,
            RIGHT_BARS_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.left_bars + params.right_bars
    }

    fn state(params: &Params) -> State {
        let window = params.left_bars + params.right_bars + 1;
        State {
            highs: RollingWindow::new(window),
            lows: RollingWindow::new(window),
            left: params.left_bars,
            upper: f64::NAN,
            lower: f64::NAN,
        }
    }
}
