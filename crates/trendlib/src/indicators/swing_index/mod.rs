use crate::core::chart::at_least;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "swing_index";

pub const LIMIT_MOVE_DEFAULT: f64 = 0.5;
pub const LIMIT_MOVE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub limit_move: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            limit_move: LIMIT_MOVE_DEFAULT,
        }
    }
}

/// Wilder's swing index of one bar against the bar before it.
pub(crate) fn swing(bar: [f64; 4], was: [f64; 2], limit: f64) -> f64 {
    let [open, high, low, close] = bar;
    let [was_open, was_close] = was;
    let up = (high - was_close).abs();
    let down = (low - was_close).abs();
    let span = (high - low).abs();
    let body = (was_close - was_open).abs();
    let range = if up >= down && up >= span {
        up - 0.5 * down + 0.25 * body
    } else if down >= up && down >= span {
        down - 0.5 * up + 0.25 * body
    } else {
        span + 0.25 * body
    };
    // Only a bar that did not move from a bar that did not move has no range.
    if range == 0.0 {
        return 0.0;
    }
    let moved = (close - was_close) + 0.5 * (close - open) + 0.25 * (was_close - was_open);
    50.0 * (moved / range) * (up.max(down) / limit)
}

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<[f64; 2]>,
    limit: f64,
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let out = self.preview(bar);
        self.previous = Some([bar[0], bar[3]]);
        out
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.previous.map(|was| [swing(bar, was, self.limit)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SwingIndex;

pub type SwingIndexStream = BarStream<SwingIndex, 4, 1>;

impl Kernel<4, 1> for SwingIndex {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["swing_index"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "limit_move", params.limit_move, LIMIT_MOVE_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            limit: params.limit_move,
        }
    }
}
