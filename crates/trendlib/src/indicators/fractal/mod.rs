use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "fractal";

pub const LEFT_BARS_DEFAULT: usize = 2;
pub const LEFT_BARS_MIN: usize = 1;
pub const LEFT_BARS_MAX: usize = 100000;
pub const RIGHT_BARS_DEFAULT: usize = 2;
pub const RIGHT_BARS_MIN: usize = 1;
pub const RIGHT_BARS_MAX: usize = 100000;

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

#[derive(Clone, Debug)]
pub struct State {
    highs: RollingWindow,
    lows: RollingWindow,
    right: usize,
}

/// `100` when the bar `right` places back beats every other bar in the window.
/// The comparison is strict, so a bar that merely ties is not a swing and a
/// flat stretch marks nothing.
fn marks(held: &[f64], right: usize, want_max: bool) -> f64 {
    let middle_at = held.len() - 1 - right;
    let middle = held[middle_at];
    let beaten = held.iter().enumerate().all(|(at, value)| {
        at == middle_at
            || if want_max {
                middle > *value
            } else {
                middle < *value
            }
    });
    if beaten { 100.0 } else { 0.0 }
}

impl Step<2, 2> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 2]> {
        let full = self.highs.push(bar[0]);
        self.lows.push(bar[1]);
        full.then(|| {
            let highs: Vec<f64> = self.highs.values().collect();
            let lows: Vec<f64> = self.lows.values().collect();
            [
                marks(&highs, self.right, true),
                marks(&lows, self.right, false),
            ]
        })
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.highs.preview_is_full().then(|| {
            let highs: Vec<f64> = self.highs.preview_values(bar[0]).collect();
            let lows: Vec<f64> = self.lows.preview_values(bar[1]).collect();
            [
                marks(&highs, self.right, true),
                marks(&lows, self.right, false),
            ]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fractal;

pub type FractalStream = BarStream<Fractal, 2, 2>;

impl Kernel<2, 2> for Fractal {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 2] = ["fractal_swing_high", "fractal_swing_low"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(LEFT_BARS_MIN..=LEFT_BARS_MAX).contains(&params.left_bars) {
            return Err(TlError::param_out_of_range(
                NAME,
                "left_bars",
                params.left_bars,
                LEFT_BARS_MIN,
                LEFT_BARS_MAX,
            ));
        }
        if !(RIGHT_BARS_MIN..=RIGHT_BARS_MAX).contains(&params.right_bars) {
            return Err(TlError::param_out_of_range(
                NAME,
                "right_bars",
                params.right_bars,
                RIGHT_BARS_MIN,
                RIGHT_BARS_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.left_bars + params.right_bars
    }

    fn state(params: &Params) -> State {
        State {
            highs: RollingWindow::new(params.left_bars + params.right_bars + 1),
            lows: RollingWindow::new(params.left_bars + params.right_bars + 1),
            right: params.right_bars,
        }
    }
}
